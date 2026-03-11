# TizenClaw 멀티 에이전트 및 퍼셉션 로드맵

> **작성일**: 2026-03-11
> **참고자료**: [프로젝트 분석](ANALYSIS.md) | [시스템 설계](DESIGN.md)

---

## 1. 개요
TizenClaw가 제한된 임베디드 환경 내에서 복잡하고 장기적인 운영 워크플로우를 처리하기 위해 성숙해짐에 따라, 기존의 모놀리식 세션 기반 에이전트 접근 방식은 고급 **퍼셉션 계층(Perception Layer)** 의 지원을 받는 고도로 분산되고 안정적인 **11개의 MVP 에이전트 세트**로 전환됩니다.

이 로드맵은 해당 전환 계획을 자세히 설명합니다.

---

## 2. Phase A: MVP 에이전트 세트 구축

### 11-Agent MVP 세트
임베디드 디바이스에서의 운영 안정성을 달성하기 위해, 기존의 Orchestrator와 Skill Manager를 분리하여 7개 카테고리에 걸쳐 11개의 명확한 역할로 전문화합니다.

| 카테고리 | 에이전트 | 주요 책임 |
|----------|----------|-----------|
| **이해** | `Input Understanding Agent` | 모든 채널의 사용자 입력을 단일한 인텐트(Intent) 구조로 표준화. |
| **인식** | `Environment Perception Agent` | 이벤트 버스를 구독하여 공통 상태 스키마(Common State Schema) 유지. |
| **기억** | `Session / Context Agent` | 단기 기억(현재 작업), 장기 기억(사용자 선호), 에피소드 기억 관리. |
| **판단** | `Planning Agent` (오케스트레이터) | 퍼셉션과 Capability Registry 기반 목표를 논리적 단계로 분해. |
| **실행** | `Action Execution Agent` | OCI 컨테이너 스킬 및 Action Framework 명령 호출 수행. |
| **보호** | `Policy / Safety Agent` | 실행 전 계획을 가로채어 정책(샌드박스 제한 등) 시행. |
| **유틸리티** | `Knowledge Retrieval Agent` | 시맨틱 검색용 SQLite RAG 저장소 인터페이스. |
| **모니터링** | `Health Monitoring Agent` | 메모리 압박(PSS), 데몬 업타임, 컨테이너 건전성 등 모니터링 관리. |
| | `Recovery Agent` | 구조적 실패 분석(예: DNS Timeout) 및 폴백 또는 오류 교정 시도. |
| | `Logging / Trace Agent` | 디버깅 및 감사 기록을 위한 컨텍스트 중앙화 수행. |

*(기존의 `Skill Manager` 에이전트는 향후 RPK 기반 도구 제공 체계가 완성됨에 따라 실행 및 복구 레이어로 흡수될 예정입니다.)*

---

## 3. Phase B: 퍼셉션(Perception) 아키텍처 구현

견고한 멀티 에이전트 시스템은 고품질의 퍼셉션 기능에 의존합니다. TizenClaw의 퍼셉션 레이어는 다음의 주요 원칙에 따라 설계됩니다:

### 3.1 공통 상태 스키마 (Common State Schema)
비정형 `/proc` 데이터나 파편화된 로그 정보를 정규화된 형태의 JSON 스키마로 지속 제공합니다:
- `DeviceState`: 활성화 기능 (디스플레이, BT, WiFi), 모델 구분, 장치명.
- `RuntimeState`: 네트워크 상태, 메모리 압박 상태, 모드 등.
- `UserState`: 로캘, 환경 설정 특성, 사용자 권한 정보.
- `TaskState`: 현재 다루고 있는 목표, 진행 단계, 빈 의도(Intent slots).

### 3.2 Capability Registry 및 함수 계약(Function Contracts)
동적으로 적용된 RPK 플러그인, CLI 툴 및 내장 스킬 모두가 입력/출력 구조, 부작용(Side Effects), 재시도 정책, 요구 권한이 구체화된 함수 계약과 함께 `Capability Registry`에 등록되어 플래닝 에이전트가 반영합니다.

### 3.3 이벤트 버스 (Event-Driven Updates)
CPU 점유율 낭비를 막기 위해 무의미한 폴링 대신 세분화된 이벤트(`sensor.changed`, `network.disconnected`, `user.command.received`, `action.failed` 등)를 발생시켜 상태 정보를 새롭게 갱신합니다.

### 3.4 메모리 구조의 분리 관리
- *단기(Short-term)*: 직전 커맨드 이력, 현재 대화 기록, 즉각적인 동작 실패 원인.
- *장기(Long-term)*: 사용자별 환경 선호 설정, 일반적 디바이스 사용 프로파일.
- *에피소드(Episodic)*: 특정 운영 환경에 따른 스킬 동작 성공 및 실패 내역 기반.

### 3.5 임베디드 핵심 설계 원칙
- **선택적 컨텍스트 주입**: LLM에 필요한 상태만 선별하여 인젝션. 예를 들어, 무작위로 축적된 문자 데이터 1000줄보다 `[network: disconnected, reason: dns_timeout]` 과 같이 처리 가능한 형태로 스크리닝됩니다.
- **인식 단계와 실행 단계의 분리**: 퍼셉션 에이전트는 단순히 인지하여 상태를 확보하고 실행 에이전트는 의존성 없이 상태만 변형시킵니다.
- **확신도(Confidence) 부여 방식**: 의도 파악과 객체 분석 과정에서 생성된 점수(예: `confidence: 0.82`)를 수반하여 확신 단계가 모호할 시 명확한 추가 질문 체계 허용.

---

## 4. Phase C: RPK 도구 배포 확장

기본적인 퍼셉션 구축이 완료된 이후, `RPK Tool Distribution`을 구조화합니다.

Tizen 리소스 패키지(RPK)의 배포 구조:
- 샌드박싱된 Python 스킬 번들링
- 호스트 / 컨테이너 지원 CLI 기반 분석 개발 도구 관리 형태 제공

앞서 기술된 패키지 방식은 시스템 데몬 컴파일 및 빌딩에 영향없이 동적인 상태로 `Capability Registry`를 채우게 됩니다.
