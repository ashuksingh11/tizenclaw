# RAG 시스템 (검색 증강 생성)

TizenClaw는 LLM 백엔드에 독립적인 **온디바이스 임베딩** 시스템을 사용하여 RAG를 구현합니다. 어떤 LLM(Gemini, OpenAI, Ollama 등)이 활성화되어 있더라도 일관된 시맨틱 검색 결과를 보장합니다.

## 아키텍처

```
┌─────────────────────────────────┐    ┌─────────────────────────────────┐
│     빌드 타임 (호스트 PC)         │    │      런타임 (디바이스)           │
│                                 │    │                                 │
│  Python + sentence-transformers │    │  C++ + ONNX Runtime (dlopen)   │
│  all-MiniLM-L6-v2 → 384차원     │    │  all-MiniLM-L6-v2 → 384차원    │
│           ↓                     │    │           ↓                     │
│  tizen_api.db   (43 MB)        │    │  쿼리 임베딩 생성                │
│  tizen_guide.db (19 MB)        │    │  코사인 유사도 검색              │
│           ↓                     │    │           ↓                     │
│  RPM 설치 ─────────────────────────→ /opt/usr/share/tizenclaw/rag/    │
└─────────────────────────────────┘    └─────────────────────────────────┘
```

## 컴패니언 프로젝트: tizenclaw-rag

RAG 자산은 **별도 프로젝트**로 관리됩니다: [tizenclaw-rag](https://github.com/hjhun/tizenclaw-rag)

이 프로젝트는 다음을 포함하는 독립적인 RPM을 생성합니다:

| 컴포넌트 | 설치 경로 | 크기 |
|---------|----------|------|
| 지식 데이터베이스 | `/opt/usr/share/tizenclaw/rag/` | ~62 MB |
| ONNX Runtime | `/opt/usr/share/tizenclaw/lib/` | ~16 MB |
| 임베딩 모델 | `/opt/usr/share/tizenclaw/models/all-MiniLM-L6-v2/` | ~90 MB |

### 지식 데이터베이스

| 데이터베이스 | 소스 | 파일 수 | 청크 수 |
|------------|------|-------:|-------:|
| `tizen_api.db` | Native C-API Doxygen (HTML) | 437 | 10,915 |
| `tizen_guide.db` | Native Guides (Markdown) | 302 | 4,770 |

### 빌드 및 배포

```bash
# 방법 A: 자동 (deploy.sh 사용)
# deploy.sh가 ../tizenclaw-rag를 자동 감지하여 함께 빌드합니다
./deploy.sh

# 방법 B: 수동
cd ../tizenclaw-rag
gbs build -A x86_64 --include-all
```

### RAG 데이터베이스 재생성

소스 문서에서 지식 데이터베이스를 다시 빌드해야 하는 경우:

```bash
cd ../tizenclaw-rag

# tizen-docs가 없으면 GitHub에서 자동 다운로드
./scripts/setup_docs.sh

# 로컬 임베딩으로 데이터베이스 빌드 (API 키 불필요)
pip3 install sentence-transformers
./scripts/build_knowledge_db.sh
```

## 온디바이스 임베딩 모듈

C++ 임베딩 모듈(`src/tizenclaw/embedding/`)은 메인 tizenclaw 프로젝트에 포함되어 있습니다:

| 파일 | 설명 |
|------|------|
| `wordpiece_tokenizer.{hh,cc}` | BERT 호환 WordPiece 토크나이저 |
| `on_device_embedding.{hh,cc}` | ONNX Runtime 추론 (dlopen), `all-MiniLM-L6-v2` 모델 |
| `onnxruntime_c_api.h` | 공식 ONNX Runtime C API 헤더 (v1.20.1) |

### 동작 원리

1. **토큰화** — WordPiece 어휘(`vocab.txt`, 30,522 토큰)로 입력 텍스트 토큰화
2. **추론** — ONNX Runtime이 `all-MiniLM-L6-v2` 모델을 실행하여 토큰별 hidden state 생성
3. **풀링** — Attention mask를 사용한 Mean pooling으로 384차원 임베딩 생성
4. **정규화** — 코사인 유사도 호환을 위한 L2 정규화
5. **검색** — SQLite에 사전 계산된 임베딩에 대한 brute-force 코사인 유사도 검색

### EmbeddingStore 다중 DB 지원

`EmbeddingStore`는 여러 지식 데이터베이스를 동시에 연결합니다:

```
embeddings.db (메인 스토어)
  ↓ ATTACH
knowledge_0 → tizen_guide.db
knowledge_1 → tizen_knowledge.db
knowledge_2 → tizen_api.db
```

쿼리는 연결된 모든 데이터베이스를 검색하여 가장 유사한 top-k 결과를 반환합니다.

## 지원 아키텍처

| 아키텍처 | ONNX Runtime | 상태 |
|:-------:|:---:|:---:|
| x86_64 | ✅ 사전빌드 | 완전 지원 |
| aarch64 | ✅ 사전빌드 | 완전 지원 |
| armv7l | ✅ 크로스컴파일 | 완전 지원 |

armv7l 라이브러리는 `arm-linux-gnueabihf-gcc`를 사용하여 소스에서 크로스컴파일됩니다. 재빌드:
```bash
cd ../tizenclaw-rag
bash scripts/build_ort_armv7l.sh ~/path/to/onnxruntime
```
