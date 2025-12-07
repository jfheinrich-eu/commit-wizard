# AI Configuration (LEGACY DOCUMENTATION)

> **⚠️ NOTICE:** This documentation is **OUTDATED** and kept for historical reference only.
>
> **Current Version:** commit-wizard now uses **GitHub Copilot CLI** for AI features.
> No API tokens or environment variables are required.
>
> See the main [README.md](../README.md) for current setup instructions.

---

## Historical Context (Pre-Copilot CLI Integration)

This document describes the old approach using direct API calls to GitHub Models and OpenAI.
This implementation was replaced in December 2025 due to:

- HTTP API access blocked by GitHub in certain environments
- Complex token management
- Regional availability issues
- Need for multiple API key configurations

### Old Architecture (Deprecated)

Previously, commit-wizard supported:

1. **GitHub Models API** (required `GITHUB_TOKEN` with `read:user` scope)
2. **OpenAI API** (required `OPENAI_API_KEY`)
3. Automatic fallback between providers

### Why This Changed

The HTTP API approach had several issues:

- `models.github.com` was not accessible in some environments (Codespaces, corporate networks)
- DNS resolution failures
- Regional restrictions
- Complex authentication flow
- API rate limits

### Current Solution

**GitHub Copilot CLI** provides:

- ✅ Built-in authentication via `gh auth` and `copilot auth`
- ✅ Works in all environments where GitHub CLI works
- ✅ No environment variables needed
- ✅ Interactive authentication flow
- ✅ Automatic token refresh
- ✅ Better error handling

---

## Legacy Documentation Below (For Reference Only)

<details>
<summary>Click to expand old documentation</summary>

### Old Option 1: GitHub Models API

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
```

**Scope:** `read:user`
**Endpoint:** `https://models.github.com/chat/completions`

### Old Option 2: OpenAI API

```bash
export OPENAI_API_KEY="sk-xxxxxxxxxxxx"
```

**Endpoint:** `https://api.openai.com/v1/chat/completions`
**API Key:** https://platform.openai.com/api-keys

### Old Automatic Selection Logic

1. If `GITHUB_TOKEN` set → GitHub Models API
2. If `OPENAI_API_KEY` set → OpenAI API
3. Otherwise → Error message

</details>

### Mögliche Ursachen

1. **Codespaces/Container**: models.github.com möglicherweise blockiert
2. **Regionale Einschränkungen**: Nicht in allen Ländern verfügbar
3. **Beta-Feature**: Noch nicht öffentlich verfügbar
4. **Firewall/Proxy**: Netzwerk-Einschränkungen

## Empfohlene Konfiguration

### Für Entwicklung (lokal)

```bash
# .env Datei
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

### Für Codespaces/CI

```bash
# .env Datei
OPENAI_API_KEY=sk-xxxxxxxxxxxx
```

### Für beide Umgebungen

```bash
# .env Datei
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
OPENAI_API_KEY=sk-xxxxxxxxxxxx
```

commit-wizard versucht automatisch GitHub Models API first, fällt auf OpenAI zurück wenn nötig.

## Token-Test aktualisieren

Der `test-token` Command wurde erweitert:

```bash
# Test beide Token-Optionen
commit-wizard test-token

# Zeigt:
# ✓ GITHUB_TOKEN gefunden → Teste GitHub Models API
# ✗ GitHub Models nicht erreichbar → Fallback auf OpenAI
# ✓ OPENAI_API_KEY gefunden → Teste OpenAI API
# ✅ OpenAI API funktioniert!
```

## OpenAI API Key erstellen

1. Gehe zu https://platform.openai.com/
2. Sign in / Sign up
3. Gehe zu API Keys: https://platform.openai.com/api-keys
4. "Create new secret key"
5. Kopiere den Key (beginnt mit `sk-`)
6. Setze: `export OPENAI_API_KEY="sk-..."`

**Wichtig:** OpenAI API kostet Geld! Aber:
- Sehr günstig für commit messages (~$0.0001 pro Message)
- ~~Erste $5 free credit für neue Accounts~~ (Free Trial wurde 2023 eingestellt)
- Du musst **Billing aktivieren** und eine Zahlungsmethode hinterlegen
- Setze usage limits in OpenAI Dashboard um Kosten zu kontrollieren

### OpenAI Free Plan / Credits Problem

**Symptom:** API funktioniert im Test, aber keine Usage sichtbar im Portal

**Mögliche Ursachen:**

1. **Keine Free Trial mehr**: OpenAI hat Free Trial Ende 2023 eingestellt
   - Neue Accounts brauchen Zahlungsmethode
   - Alte Free Trial Credits sind abgelaufen

2. **Veralteter API Key**: Key funktioniert noch, aber Account inaktiv
   - Prüfe: [OpenAI Billing Settings](https://platform.openai.com/settings/organization/billing)
   - Aktiviere Billing mit Kreditkarte/PayPal

3. **Usage Reporting Delay**: Usage kann 5-10 Minuten verzögert angezeigt werden
   - Warte kurz und refreshe die Usage-Seite

4. **Test-Caching**: Sehr kleine Requests werden manchmal gecacht
   - Mache einen größeren Test-Request (siehe unten)

**So prüfst du ob Credits verfügbar sind:**

```bash
# Manueller Test mit größerem Request
curl -s https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Write 100 words about AI"}],
    "max_tokens": 150
  }' | jq -r 'if .error then "ERROR: \(.error.message)" else "SUCCESS - \(.usage.total_tokens) tokens used" end'
```

**Wenn "ERROR: You exceeded your current quota":**

- Gehe zu [OpenAI Billing Settings](https://platform.openai.com/settings/organization/billing)
- Füge Zahlungsmethode hinzu
- Lade Credits auf (Minimum $5)

**Kosten für typische Nutzung:**

- gpt-4o-mini: ~$0.00015 pro 1000 tokens
- Durchschnittliche Commit Message: ~200 tokens
- **Kosten pro Commit:** ~$0.00003 (3 Hundertstel Cent!)
- 1000 Commits ≈ $0.30 (30 Cent)

## Zusammenfassung der Scope-Frage

**Für GitHub Models API:**

- ✅ `read:user` Scope reicht aus
- ❌ Problem ist NICHT der Scope
- ❌ Problem ist Netzwerk/DNS/Availability

**Keine zusätzlichen Scopes nötig!**

Die ursprüngliche Annahme, dass Models API spezielle Scopes braucht, war falsch.

## Fehlerdiagnose

### "No API token found"

```bash
# Prüfe was gesetzt ist
env | grep -E "(GITHUB_TOKEN|OPENAI_API_KEY)"

# Setze mindestens einen
export OPENAI_API_KEY="sk-..."
```

### "Failed to connect to GitHub Models API"

```bash
# Das ist OK! Verwende OpenAI als Fallback
export OPENAI_API_KEY="sk-..."
commit-wizard --ai
```

### "AI API returned error 401"

**GitHub Token:**

- Token abgelaufen → Neu erstellen
- Token revoked → Neu erstellen

**OpenAI Key:**

- Key ungültig → Neu erstellen
- Account gesperrt → OpenAI Support

### "AI API returned error 429"

- Rate limit überschritten
- Warte 1 Minute und versuche erneut
- Bei OpenAI: Prüfe Billing/Limits

## Verwendung

```bash
# Mit GitHub Models (wenn verfügbar)
export GITHUB_TOKEN="ghp_..."
commit-wizard --ai

# Mit OpenAI (immer funktioniert)
export OPENAI_API_KEY="sk-..."
commit-wizard --ai

# Mit beiden (automatischer Fallback)
export GITHUB_TOKEN="ghp_..."
export OPENAI_API_KEY="sk-..."
commit-wizard --ai
```

## Kosten-Vergleich

| API | Kosten | Verfügbarkeit | Speed |
|-----|--------|---------------|-------|
| GitHub Models | Kostenlos* | Eingeschränkt | Schnell |
| OpenAI | ~$0.0001/msg | Global | Sehr schnell |

*GitHub Models möglicherweise Beta/Limited Access

Für Production: **OpenAI API empfohlen** (zuverlässig, global, sehr günstig)
