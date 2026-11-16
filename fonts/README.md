# fonts/ — ملفات الخطوط المرفقة بالمستودع

> **الحالة:** ✅ مضاف إلى المستودع (2026-11-15). جاهز للرفع إلى استضافتك الخاصة.
> **الترخيص:** كل الخطوط هنا **OFL-1.1** — وملف `LICENSE` (نص OFL) موجود بجانب كل حزمة.
> **الفهرس الكامل بالبصمات:** `fonts/manifests/index.json`

---


## الترخيص

خطوط `fonts/vendor/` **ليست** تحت AGPL ولا MIT. كل عائلة خط تحتفظ بترخيصها (SIL OFL 1.1)، ولكل مجلد ملف `LICENSE` خاص به.

- البرنامج: **AGPL-3.0-or-later** — `../LICENSE`
- أدوات `tools/`: **MIT** — `../LICENSE-TOOLS`


## 1. البنية

```
fonts/
├── vendor/
│   ├── inter/                    2 ملفًا   · 0.05 MB  · مدمج في المُثبِّت
│   ├── ibm-plex-sans-arabic/     2 ملفًا   · 0.08 MB  · مدمج
│   ├── pt-sans/                  4 ملفات  · 0.14 MB  · مدمج
│   ├── noto-sans-devanagari/     2 ملفًا   · 0.10 MB  · مدمج
│   ├── noto-sans-sc/           204 ملفًا  · 6.83 MB  · CDN
│   ├── noto-sans-tc/           218 ملفًا  · 6.34 MB  · CDN
│   └── noto-sans-jp/           250 ملفًا  · 7.82 MB  · CDN
└── manifests/
    ├── index.json                ← الفهرس الموحّد (كل الحزم + البصمات)
    ├── fontpack-zh-Hans.json     ← تفصيل لكل ملف
    ├── fontpack-zh-Hant.json
    └── fontpack-ja.json
```

| النوع | الحجم | أين يُستخدم |
|-------|-------|-------------|
| **مدمج (bundled)** | **0.37 MB** | داخل المُثبِّت — يعمل بلا إنترنت من أول لحظة |
| **CDN (lazy)** | **20.99 MB** | تُرفع لموقعك، وتُنزَّل عند اختيار لغة صينية/يابانية |
| **الإجمالي** | **21.36 MB** | |

---

## 2. ما ترفعه إلى موقعك

**ارفع مجلدات `vendor/noto-sans-sc`, `vendor/noto-sans-tc`, `vendor/noto-sans-jp` كما هي** إلى المسار:

```
https://<your-domain>/fonts/v1/
├── noto-sans-sc/
├── noto-sans-tc/
├── noto-sans-jp/
└── index.json      ← من fonts/manifests/index.json
```

**لا ترفع** `inter` و`ibm-plex-sans-arabic` و`pt-sans` و`noto-sans-devanagari` — مدمجة في التطبيق أصلًا (0.37MB).

**ترويسات مطلوبة:**
```
Content-Type: font/woff2
Cache-Control: public, max-age=31536000, immutable
Access-Control-Allow-Origin: *
Content-Encoding: identity
```

---

## 3. كيف يتحقق التطبيق

`fonts/manifests/index.json` يحوي لكل ملف: **الاسم + الحجم بالبايت + SHA-256**. التطبيق:

1. يقرأ الفهرس (نسخة موقّعة داخل الإصدار).
2. يحمّل النطاقات المطلوبة فقط عبر `unicode-range`.
3. يتحقق من `sha256` و`bytes` لكل ملف.
4. **أي اختلاف → يرفض الملف** ويبقى على خط النظام (§4 في `docs/cdn-fonts.md`).

---

## 4. ملاحظات تقنية

| # | الملاحظة |
|---|----------|
| 1 | **PT Sans لا يملك وزن 600** — استخدمنا **700** كبديل لـ Semibold. مسجَّل في `index.json`. |
| 2 | Inter وIBM Plex Sans Arabic وNoto Sans Devanagari: وزن **600 موجود** ومستخدم كما هو. |
| 3 | ملفات `files/` الأصلية تضم كل الأوزان (100–900) وصيغتَي `woff`/`woff2` = **217 MB**. نحن أخذنا **400+600 بـ woff2 فقط** = 21 MB. |
| 4 | كل ملف مرفوع هنا **مطابق بايتًا ببايت** لما هو في حزم npm الرسمية (نسخة 5.3.1 للآسيوية). |
| 5 | **لا تُعد ضغط الملفات** (gzip/brotli) — `woff2` مضغوط داخليًا. |

---

## 5. تحديث البصمات

إذا حدّثت أي حزمة لاحقًا:

```bash
npm i --no-save @fontsource/noto-sans-sc@<ver>
# ثم أعد توليد الفهرس
python3 tools/gen-font-manifests.py
```

> **ممنوع** تعديل ملف `.woff2` يدويًا أو إعادة تسميته — التطبيق سيرفضه لأن البصمة لن تطابق.
