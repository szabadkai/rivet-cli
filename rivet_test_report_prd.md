# PRD — Rivet Test Report

## Vision
A stylish, branded, zero-setup HTML test report that engineers *want* to use, managers can skim, and CI/CD systems can parse. Functional, polished, and shareable.

---

## Goals
- **Functional:** Summarize results, highlight failures, show coverage & performance.  
- **Usable:** Fast triage, easy navigation, single-file artifact.  
- **Lovable:** Rivet branding, dark theme, subtle ASCII/animations.  
- **Shareable:** Air-gapped capable, no dependencies.

---

## Functional Requirements

### Input/Output
- Input: JSON report from `rivet run`.
- Output: Single self-contained HTML file (CSS/JS embedded).

### Features
1. **Header**
   - Rivet branding & run metadata (date, duration, commit, version).
   - Status banner (✅ GREEN / ❌ RED / ⚠ WARN).

2. **Summary Dashboard**
   - Cards: total tests, passed, failed, skipped, flaky.
   - Pass rate dial, duration sparkline, coverage summary.

3. **Failure Feed**
   - Expandable cards: request/response, assertion diffs, pretty JSON.  
   - Copy-to-cURL button.

4. **Detailed Log**
   - Tree: suite → test → request.  
   - Status icons, duration, filter/search.

5. **Coverage Tab**
   - Table + heatmap: hit/missed endpoints (OpenAPI).  
   - Export JSON/CSV.

6. **Performance Tab**
   - Top slowest tests, p50/p95/p99 stats.

7. **Footer**
   - Rivet signature, ASCII confetti on all-pass.

### Interactions
- Filters: status, suite, duration.  
- Search: fuzzy by test name or URL.  
- Expand/collapse all.  
- Export buttons: JSON/CSV.

---

## Nonfunctional Requirements
- **Performance:** Handle 10k+ tests without UI lag (lazy rendering).  
- **Size:** ≤2 MB gzipped for large runs.  
- **Portability:** Opens offline, no external CDN.  
- **Accessibility:** Responsive, keyboard-friendly, WCAG AA.  
- **Theming:** Dark/light mode toggle.  
- **Branding:** Rivet ASCII logo + consistent cyan/teal/green highlights.  
- **Robustness:** Auto-disable animations in CI/non-TTY.  

---

## Non-Goals
- No hosted dashboards or run history comparison.  
- No external asset loading.

---

## Success Metrics
- 80%+ engineer satisfaction (survey).  
- CI artifact open rate >70% of failed runs.  
- Time-to-root-cause <5 min in failure scenarios.

