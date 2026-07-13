# Languages

## How to add a new language

1. Copy `en.json` -> `{code}.json` (e.g. `it.json`)
2. Translate all values
3. Add the flag and import in `index.ts`:

```ts
import it from "./it.json";

// in LANGUAGES array:
{ code: "it", nativeName: "Italiano", flag: FLAG_IT, aiTranslated: false },
```

4. If translation is AI-generated (and/or not verified by a native speaker) or incomplete, set `aiTranslated: true`

## Rules

- **Keep all keys** — do not rename or remove them
- **Keep placeholders** — `{time}`, `{version}`, `{name}` etc. are filled by the app
- **Match casing** — if English has `"STOP"`, keep it uppercase in your translation too
- **Empty values** — if a value is `""`, the app will show English fallback

## Flags

Flag SVGs are from https://github.com/lipis/flag-icons/blob/main/flags/4x3/.
