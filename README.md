# 🛒 E-commerce Simplificado — Desafio LAPES 2026

**Candidato:** Benjamin Yuji Suzuki
**Trilha:** Desenvolvimento — Mini E-commerce
**Formato:** Individual

**Contato:**
- E-mail: benjamin24070067@aluno.cesupa.br
- WhatsApp: (91) 99208-7892

---

## Por que Rust?

Rust oferece segurança de memória sem garbage collector, tipagem forte que elimina classes inteiras de bugs em tempo de compilação, e performance próxima de C. Para um sistema de e-commerce onde consistência de dados e controle de concorrência são requisitos explícitos, o compilador do Rust atua como um primeiro revisor de código — se compilou, grande parte dos erros de lógica concorrente já foram eliminados. A máquina de estados do pedido (`PENDING → PAID → SHIPPED → DELIVERED`) é modelada como um enum Rust, tornando transições inválidas literalmente impossíveis de compilar. O controle de overselling no checkout usa transações atômicas no PostgreSQL combinadas com as garantias do type system do Rust.

---

## Stack Tecnológico

### Backend

| Crate | Função |
|---|---|
| `axum` | Framework web async, construído sobre Tokio. Roteamento, middleware. |
| `tokio` | Runtime assíncrono. Base de toda a stack de I/O. |
| `sqlx` | Queries SQL async para PostgreSQL com verificação em tempo de compilação. Migrations embutidas. |
| `redis` + `deadpool-redis` | Cache de produtos com invalidação. Pool de conexões async. |
| `jsonwebtoken` | Emissão e validação de JWT para autenticação. |
| `argon2` | Hash de senhas. Algoritmo moderno e seguro. |
| `tower-http` | Middleware CORS e tracing para requisições HTTP. |
| `tracing` + `tracing-subscriber` | Logs estruturados em JSON. |
| `serde` + `serde_json` | Serialização e deserialização de JSON. |
| `validator` | Validação de inputs nas bordas da API. |
| `reqwest` | Cliente HTTP para integração com gateway de pagamento. |
| `uuid` | Geração de UUIDs para identificadores de entidades. |
| `chrono` | Manipulação de datas e timestamps. |

**Banco de dados:** PostgreSQL 16
**Cache:** Redis 7
**Migrations:** `sqlx migrate` (versionadas em `migrations/*.sql`)

### Frontend

| Ferramenta | Função |
|---|---|
| `Leptos 0.7` | Framework web Rust compilado para WebAssembly. Modo CSR (Client-Side Rendering). |
| `Trunk` | Bundler WASM para Rust. Empacota e serve o frontend. |
| `TailwindCSS` | Estilização via classes utilitárias. |

Frontend 100% em Rust (`.rs`) usando a macro `view!` do Leptos compilado para WebAssembly. Sem JavaScript ou TypeScript.

### Infraestrutura

| Ferramenta | Função |
|---|---|
| `Docker` + `Docker Compose` | Orquestra API, PostgreSQL e Redis. |
| `GitHub Actions` | Pipeline de CI: build, testes, lint (`clippy`). |

---

## Arquitetura

```
┌─────────────────────────────────────────────────────┐
│                     Cliente                          │
│              Leptos CSR (Rust → WASM)               │
└──────────────────────┬──────────────────────────────┘
                       │ HTTP / REST (JSON)
┌──────────────────────▼──────────────────────────────┐
│                  Axum (Backend)                      │
│  Auth │ Catálogo │ Carrinho │ Checkout │ Cupons      │
└────────┬──────────────────────────────┬─────────────┘
         │                              │
┌────────▼────────┐          ┌──────────▼──────────┐
│   PostgreSQL    │          │        Redis         │
│  (dados + stock)│          │  (cache + sessions)  │
└─────────────────┘          └──────────────────────┘
```

---

## Domínios Implementados

- **Autenticação & Usuários** — Registro, login, JWT, roles `admin` e `customer`, proteção de rotas.
- **Catálogo de Produtos** — CRUD completo, busca com filtros (categoria, preço, nome), paginação, cache Redis com invalidação.
- **Carrinho de Compras** — Carrinho persistido por usuário, validação de estoque ao adicionar e no checkout.
- **Checkout & Pedidos** — Reserva atômica de estoque (sem overselling), máquina de estados, cancelamento com devolução de estoque, integração com gateway de pagamento.
- **Cupons de Desconto** — Percentual ou valor fixo, validade por data, uso único por usuário, valor mínimo de pedido.
- **Health Check** — Endpoint de saúde para monitoramento.

---

## Como Rodar

### Pré-requisitos

- [Rust](https://rustup.rs/) (edition 2024, stable)
- [Docker](https://www.docker.com/) e Docker Compose
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### Desenvolvimento local (recomendado)

```bash
# Clone o repositório
git clone https://github.com/Benjamin-Yuji-Suzuki/processo-seletivo-2026.git
cd processo-seletivo-2026

# Rode o script de dev (sobe PostgreSQL, Redis, API e Frontend)
./dev.sh
```

**Portas:**
- API: `http://localhost:8099`
- Frontend: `http://localhost:8081`
- Swagger: `http://localhost:8099/swagger-ui/`

### Docker (produção)

```bash
docker compose up --build
```

A API sobe com PostgreSQL e Redis. O frontend (Trunk/WASM) roda separadamente.

### Setup manual passo a passo

```bash
# 1. Suba PostgreSQL e Redis
docker compose up -d postgres redis

# 2. Configure o .env
cp .env.example .env

# 3. Rode a API
cargo run --bin api

# 4. Em outro terminal, suba o frontend
cd frontend
trunk serve --port 8081
```

---

## Testes

```bash
# Todos os testes
cargo test

# Testes de integração (requer banco rodando)
cargo test --test integration

# Com output detalhado
cargo test -- --nocapture
```

Os testes cobrem os fluxos críticos: checkout completo, concorrência de estoque (requisições simultâneas para o último item), e validação de cupons.

---

## Estrutura do Repositório

```
.
├── api/                      # Backend Axum
│   ├── src/
│   │   ├── auth/             # Autenticação e autorização
│   │   ├── catalog/          # Catálogo de produtos
│   │   ├── cart/             # Carrinho de compras
│   │   ├── checkout/         # Checkout e pedidos
│   │   ├── coupons/          # Cupons de desconto
│   │   ├── health.rs         # Health check endpoint
│   │   ├── lib.rs            # Módulos e app router
│   │   ├── main.rs           # Entrypoint
│   │   ├── models.rs         # Modelos de domínio
│   │   ├── error.rs          # Tratamento de erros
│   │   └── state.rs          # Estado compartilhado (AppState)
│   └── Cargo.toml
├── frontend/                 # Frontend Leptos CSR
│   ├── src/
│   │   ├── components/       # Componentes reutilizáveis
│   │   ├── pages/            # Páginas (admin, cart, etc.)
│   │   └── lib.rs
│   └── Cargo.toml
├── migrations/               # SQLx migrations
├── tests/                    # Testes de integração
├── .github/workflows/        # CI/CD GitHub Actions
├── dev.sh                    # Script de desenvolvimento
├── docker-compose.yml        # Orquestração local
├── Dockerfile                # Build multi-stage da API
├── Cargo.toml                # Workspace raiz
└── README.md
```

---

## Decisões Técnicas

**Por que Axum e não Actix-web?** Axum é construído sobre Tower, o que torna middleware composável e reutilizável. A integração com o ecossistema Tokio é nativa.

**Por que SQLx e não Diesel ou SeaORM?** SQLx valida as queries SQL em tempo de compilação sem precisar de um ORM completo. Mantém o SQL legível e explícito, facilitando otimizações e auditoria.

**Por que Leptos CSR e não SSR?** O frontend foi implementado em modo Client-Side Rendering com Trunk como bundler, simplificando o deploy e eliminando a necessidade de um servidor Node ou de SSR em produção. A comunicação com a API é via REST puro.

**Controle de concorrência:** O overselling é prevenido com `SELECT ... FOR UPDATE` dentro de uma transação PostgreSQL. Duas requisições simultâneas para o último item resultam em uma transação esperando a outra terminar — a segunda recebe erro de estoque insuficiente.

**Cache:** Produtos são cacheados no Redis com TTL de 5 minutos. Qualquer operação de criação, edição ou remoção invalida a chave correspondente imediatamente.

---

## Variáveis de Ambiente

```env
DATABASE_URL=postgres://ben:1234@localhost:5432/lapes_ecommerce
REDIS_URL=redis://localhost:6379
JWT_SECRET=sua_chave_secreta_aqui
JWT_EXPIRES_IN=24h
PAYMENT_GATEWAY_URL=https://api.sandbox.gateway.com/
PAYMENT_GATEWAY_KEY=chave_do_gateway
RUST_LOG=lapes_ecommerce_api=info,tower_http=info
```

---

*Desafio LAPES 2026 — Benjamin Yuji Suzuki*
