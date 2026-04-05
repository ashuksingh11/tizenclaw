# TizenClaw CLI PinchBench 설정 가이드

이 문서는 `tizenclaw-cli`로 `llm_config.json`을 관리하면서
PinchBench 실행에 필요한 Anthropic/Gemini 설정을 맞추는 방법을
설명합니다. OpenClaw 또는 ZeroClaw에서 `config set/get`을 쓰던
흐름과 비슷하게 사용할 수 있도록 정리했습니다.

## 전제

- `tizenclaw` daemon이 실행 중이어야 합니다.
- `tizenclaw-cli`가 PATH에 있어야 합니다.
- 런타임 설정은 daemon의 `llm_config.json`에 저장됩니다.
- `tizenclaw-cli config set`은 기본적으로 문자열을 저장합니다.
- 숫자, 배열, 객체, 불리언을 저장할 때는 `--strict-json`을
  사용해야 합니다.

## 1. 현재 설정 확인

전체 설정:

```bash
tizenclaw-cli config get
```

특정 값만 확인:

```bash
tizenclaw-cli config get active_backend
tizenclaw-cli config get backends.anthropic.model
tizenclaw-cli config get backends.gemini.model
tizenclaw-cli config get benchmark.pinchbench.target.score
```

## 2. Anthropic 설정

Anthropic을 기본 백엔드로 사용:

```bash
tizenclaw-cli config set active_backend anthropic
```

Anthropic 모델 설정:

```bash
tizenclaw-cli config set \
  backends.anthropic.model \
  claude-sonnet-4-20250514
```

Anthropic API 키 설정:

```bash
tizenclaw-cli config set \
  backends.anthropic.api_key \
  sk-ant-api03-...
```

Temperature 설정:

```bash
tizenclaw-cli config set \
  backends.anthropic.temperature \
  0.7 \
  --strict-json
```

최대 출력 토큰 설정:

```bash
tizenclaw-cli config set \
  backends.anthropic.max_tokens \
  4096 \
  --strict-json
```

## 3. Gemini 설정

Gemini를 기본 백엔드로 사용:

```bash
tizenclaw-cli config set active_backend gemini
```

Gemini 모델 설정:

```bash
tizenclaw-cli config set \
  backends.gemini.model \
  gemini-2.5-flash
```

Gemini API 키 설정:

```bash
tizenclaw-cli config set \
  backends.gemini.api_key \
  AIza...
```

Temperature 설정:

```bash
tizenclaw-cli config set \
  backends.gemini.temperature \
  0.7 \
  --strict-json
```

최대 출력 토큰 설정:

```bash
tizenclaw-cli config set \
  backends.gemini.max_tokens \
  4096 \
  --strict-json
```

## 4. fallback 백엔드 설정

Anthropic 우선, Gemini fallback:

```bash
tizenclaw-cli config set \
  fallback_backends \
  '["gemini"]' \
  --strict-json
```

Gemini 우선, Anthropic fallback:

```bash
tizenclaw-cli config set \
  fallback_backends \
  '["anthropic"]' \
  --strict-json
```

## 5. PinchBench용 실제 토큰 수 기록

PinchBench 비교 기록을 위해 실제 토큰 수를
`llm_config.json`에 남길 수 있습니다.

```bash
tizenclaw-cli config set \
  benchmark.pinchbench.actual_tokens.prompt \
  18234 \
  --strict-json

tizenclaw-cli config set \
  benchmark.pinchbench.actual_tokens.completion \
  4121 \
  --strict-json

tizenclaw-cli config set \
  benchmark.pinchbench.actual_tokens.total \
  22355 \
  --strict-json
```

한 번에 확인:

```bash
tizenclaw-cli config get benchmark.pinchbench.actual_tokens
```

## 6. PinchBench 목표 결과 기록

원하는 점수나 비교 목표도 같은 파일에 함께 저장할 수 있습니다.

목표 점수:

```bash
tizenclaw-cli config set \
  benchmark.pinchbench.target.score \
  0.85 \
  --strict-json
```

대상 suite:

```bash
tizenclaw-cli config set \
  benchmark.pinchbench.target.suite \
  all
```

비교 메모:

```bash
tizenclaw-cli config set \
  benchmark.pinchbench.target.summary \
  "match openclaw anthropic baseline"
```

확인:

```bash
tizenclaw-cli config get benchmark.pinchbench.target
```

## 7. 설정 반영

`active_backend`, `backends.*` 경로는 `config set` 시 daemon이 즉시
재적용합니다. 필요하면 수동으로 다시 리로드할 수도 있습니다.

```bash
tizenclaw-cli config reload
```

## 8. 설정 삭제

더 이상 필요 없는 benchmark 메모를 제거할 때:

```bash
tizenclaw-cli config unset benchmark.pinchbench.target.summary
```

## 9. OpenClaw/ZeroClaw 식 대응 예시

OpenClaw 식:

```bash
openclaw config set agents.defaults.thinkingDefault high
openclaw config set \
  'agents.defaults.models.anthropic/claude-sonnet-4-6.params.temperature' \
  0.7 \
  --strict-json
```

TizenClaw 식:

```bash
tizenclaw-cli config set active_backend anthropic
tizenclaw-cli config set \
  backends.anthropic.temperature \
  0.7 \
  --strict-json
tizenclaw-cli config set \
  backends.anthropic.max_tokens \
  4096 \
  --strict-json
```

ZeroClaw 식 TOML 편집:

```toml
default_provider = "anthropic"
default_model = "claude-sonnet-4-6"
default_temperature = 0.7
```

TizenClaw 식 CLI 설정:

```bash
tizenclaw-cli config set active_backend anthropic
tizenclaw-cli config set \
  backends.anthropic.model \
  claude-sonnet-4-20250514
tizenclaw-cli config set \
  backends.anthropic.temperature \
  0.7 \
  --strict-json
```

## 10. 권장 점검 순서

```bash
tizenclaw-cli config get active_backend
tizenclaw-cli config get backends.anthropic
tizenclaw-cli config get backends.gemini
tizenclaw-cli config get benchmark.pinchbench
```

이 출력이 기대값과 맞으면 PinchBench 실행 전 설정 확인이 끝난
상태로 보면 됩니다.
