// SPDX-License-Identifier: MIT
// Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
// JS contract tests: a page cannot take another warning or grant deep-scan consent.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
const file = path => readFileSync(new URL('../../browser-extension/'+path,import.meta.url),'utf8');
const json = path => JSON.parse(file(path));
const manifest = json('manifest.json'), fx = json('manifest.firefox.json');
const origin = json('hosts/chromium.json').allowed_origins[0];
const id = origin.split('/')[2];
const local = 'file:///test/sample';
function event() {return {addListener(fn){this.run=fn;}};}
function harness() {
  let response={result:'clean'}, next=0;
  const menus=[], frames=[], windows=[];
  const api={runtime:{id, getURL:k=>origin+k, onInstalled:event(),onStartup:event(),onMessage:event(),
      async sendNativeMessage(host,data) {assert.equal(host,'org.odina.context');frames.push(data);return response;}},
    contextMenus:{async removeAll(){menus.length=0;},create(item){menus.push(item);},onClicked:event()},
    i18n:{getMessage:k=>json('_locales/ar/messages.json')[k].message},windows:{async create(o){windows.push(o);}}};
  vm.runInNewContext(file('background.js'),{browser:api,crypto:{randomUUID:()=>`notice-${++next}`},setTimeout:()=>0,clearTimeout:()=>{}});
  const click=(pageUrl, linkUrl, extra={})=>api.contextMenus.onClicked.run({menuItemId:'odina.inspect',pageUrl,linkUrl,...extra});
  return {api,menus,frames,windows,click,answer:r=>{response=r;}};
}
test('exact extension IDs, context menu + native messaging only; no page or network API',()=>{
  for(const m of [manifest,fx]){
    assert.deepEqual(m.permissions,['contextMenus','nativeMessaging']);
    for(const k of ['host_permissions','content_scripts','optional_permissions','externally_connectable','web_accessible_resources','action','update_url'])assert.equal(m[k],undefined,k);
    assert.match(m.content_security_policy.extension_pages,/connect-src 'none'/);
  }
  const derived=createHash('sha256').update(Buffer.from(manifest.key,'base64')).digest('hex').slice(0,32).replace(/./g,c=>'abcdefghijklmnop'[Number.parseInt(c,16)]);
  assert.equal(derived,id);assert.deepEqual(json('hosts/chromium.json').allowed_origins,[`chrome-extension://${id}/`]);
  assert.deepEqual(json('hosts/firefox.json').allowed_extensions,[fx.browser_specific_settings.gecko.id]);
  for(const f of ['background.js','notice.js']) assert.doesNotMatch(file(f),/\b(fetch|XMLHttpRequest|WebSocket|sendBeacon|EventSource|localStorage|indexedDB)\b/);
});
test('single item; remote clicks, including remote frames, never reach the host',async()=>{
  const h=harness();await h.api.runtime.onInstalled.run();await h.api.runtime.onStartup.run();assert.equal(h.menus.length,1);
  assert.equal(h.menus[0].title,'فحص بـ Odina');assert.deepEqual(Array.from(h.menus[0].contexts),['link','page']);
  for(const [page,link,extra] of [[local,'https://example.invalid'],['https://example.invalid',local],[local,local,{frameUrl:'https://example.invalid'}],[local,local,{frameId:1}],[local,'file://server.invalid/share'],[local,'file:////host/share'],[local,'file:///a?secret']]) assert.equal(await h.click(page,link,extra),'unknown');
  assert.equal(h.frames.length,0);assert.equal(h.windows.length,0);
  assert.equal(await h.click(local),'clean');h.answer({result:'unknown'});assert.equal(await h.click(local,local),'unknown');
  assert.equal(h.frames.length,2);assert.equal(h.windows.length,0);
});
test('suspect opens own window with no path or receipt in URL; handoff cannot be stolen or replayed',async()=>{
  const h=harness();h.answer({result:'suspicious',ticket:'abcdabcd01234567'});assert.equal(await h.click(local),'suspicious');
  assert.equal(h.frames.length,1);assert.equal(h.frames[0].op,'inspect');const {url}=h.windows[0];
  assert.match(url,/\/notice\.html#notice-1$/);const key=url.split('#')[1];
  const take=(sender)=>{let response;h.api.runtime.onMessage.run({op:'take',id:key},sender,r=>{response=r;});return response;};
  assert.equal(take({id,url:local}),undefined);assert.equal(take({id:'foreign',url}),undefined);
  assert.equal(take({id,url}).ticket,'abcdabcd01234567');assert.equal(take({id,url}),null);
});
