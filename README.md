# 🛒 E-commerce Simplificado — Desafio LAPES 2026

**Candidato:** Benjamin Yuji Suzuki
**Trilha:** Desenvolvimento — Mini E-commerce
**Formato:** Individual

**Contato:**
- E-mail: benjamin24070067@aluno.cesupa.br
- WhatsApp: (91) 99208-7892

---

## Por que Rust?

Porque sim.

Mas já que você perguntou: Rust oferece segurança de memória sem garbage collector, tipagem forte que elimina classes inteiras de bugs em tempo de compilação, e performance próxima de C. Para um sistema de e-commerce onde consistência de dados e controle de concorrência são requisitos explícitos, o compilador do Rust atua como um primeiro revisor de código — se compilou, grande parte dos erros de lógica concorrente já foram eliminados. A máquina de estados do pedido (`PENDING → PAID → SHIPPED → DELIVERED`) é modelada como um enum Rust, tornando transições inválidas literalmente impossíveis de compilar. O controle de overselling no checkout usa transações atômicas no PostgreSQL combinadas com as garantias do type system do Rust. Não é sobre performance — é sobre corretude.

---

## Stack Tecnológico

### Backend

| Crate | Função |
|---|---|
| `axum` | Framework web async, construído sobre Tokio. Roteamento, middleware, WebSocket. |
| `tokio` | Runtime assíncrono. Base de toda a stack de I/O. |
| `sqlx` | Queries SQL async para PostgreSQL com verificação em tempo de compilação. Migrations embutidas. |
| `redis` + `deadpool-redis` | Cache de produtos com invalidação. Pool de conexões async. |
| `jsonwebtoken` | Emissão e validação de JWT para autenticação. |
| `argon2` | Hash de senhas. Algoritmo moderno e seguro. |
| `tower_governor` | Rate limiting nos endpoints públicos (login, registro, catálogo). |
| `tracing` + `tracing-subscriber` | Logs estruturados em JSON com timestamp, método, rota, status e duração. |
| `utoipa` + `utoipa-swagger-ui` | Geração automática de documentação OpenAPI/Swagger via macros `#[derive]`. |
| `serde` + `serde_json` | Serialização e deserialização de JSON. |
| `validator` | Validação de inputs nas bordas da API. |
| `reqwest` | Cliente HTTP para integração com gateway de pagamento. |
| `uuid` | Geração de UUIDs para identificadores de entidades. |
| `chrono` | Manipulação de datas e timestamps (validade de cupons, etc). |

**Banco de dados:** PostgreSQL
**Cache:** Redis
**Migrations:** `sqlx migrate` (versionadas em `/migrations/*.sql`)

### Frontend

| Ferramenta | Função |
|---|---|
| `Leptos` | Framework web full-stack em Rust compilado para WebAssembly. SSR + hidratação no cliente. |
| `cargo-leptos` | CLI de desenvolvimento do Leptos. Hot reload, build otimizado. |
| `Trunk` | Bundler WASM para Rust. Empacota o frontend para produção. |
| `TailwindCSS` | Estilização via classes utilitárias, declaradas dentro das macros Rust. |

Todo o código do frontend é escrito em Rust puro (arquivos `.rs`) usando a macro `view!` do Leptos, que compila para WebAssembly. Não há JavaScript ou TypeScript no projeto.

### Infraestrutura

| Ferramenta | Função |
|---|---|
| `Docker` + `Docker Compose` | Orquestra localmente API, frontend, PostgreSQL e Redis. |
| `GitHub Actions` | Pipeline de CI: build, testes, lint (`clippy`) e formatação (`rustfmt`) a cada push/PR. |
| `Fly.io` | Deploy em produção. CD automático via GitHub Actions no merge para `main`. |

---

## Arquitetura

```
┌─────────────────────────────────────────────────────┐
│                     Cliente                          │
│              Leptos (Rust → WASM)                   │
└──────────────────────┬──────────────────────────────┘
                       │ HTTP / REST
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

- **Autenticação & Usuários** — Registro, login, JWT, roles `admin` e `customer`, proteção de rotas por papel.
- **Catálogo de Produtos** — CRUD completo, busca com filtros (categoria, preço, nome), paginação, cache Redis com invalidação.
- **Carrinho de Compras** — Carrinho persistido por usuário, validação de estoque ao adicionar e no checkout.
- **Checkout & Pedidos** — Reserva atômica de estoque (sem overselling), máquina de estados, cancelamento com devolução de estoque, integração com gateway de pagamento.
- **Cupons de Desconto** — Percentual ou valor fixo, validade por data, uso único por usuário, valor mínimo de pedido.

---

## Como Rodar

### Pré-requisitos

- [Rust](https://rustup.rs/) (stable)
- [Docker](https://www.docker.com/) e Docker Compose
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos): `cargo install cargo-leptos`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### Setup local

```bash
# 1. Clone o repositório
git clone https://github.com/benjaminYuji/lapes-ecommerce
cd lapes-ecommerce

# 2. Suba PostgreSQL e Redis
docker compose up -d db redis

# 3. Configure as variáveis de ambiente
cp .env.example .env
# edite o .env com suas configurações

# 4. Rode as migrations e popule o banco
cargo run --bin migrate
cargo run --bin seed

# 5. Suba o backend
cargo run --bin api

# 6. Suba o frontend (em outro terminal)
cargo leptos watch
```

A API estará disponível em `http://localhost:3000`
O frontend estará disponível em `http://localhost:3001`
A documentação Swagger em `http://localhost:3000/docs`

### Rodar tudo com Docker (recomendado)

```bash
docker compose up --build
```

Isso sobe API + frontend + PostgreSQL + Redis de uma vez. Migrations e seed rodam automaticamente.

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

## Variáveis de Ambiente

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/ecommerce
REDIS_URL=redis://localhost:6379
JWT_SECRET=sua_chave_secreta_aqui
JWT_EXPIRES_IN=24h
PAYMENT_GATEWAY_URL=https://...
PAYMENT_GATEWAY_KEY=...
RUST_LOG=info
```

---

## Decisões Técnicas

**Por que Axum e não Actix-web?** Axum é construído sobre Tower, o que torna middleware composável e reutilizável. A integração com Leptos (SSR) também é nativa no ecossistema Axum + Tokio.

**Por que SQLx e não Diesel ou SeaORM?** SQLx valida as queries SQL em tempo de compilação sem precisar de um ORM completo. Isso mantém o SQL legível e explícito, o que facilita otimizações e é mais fácil de auditar.

**Por que Leptos e não Yew ou Dioxus?** Leptos oferece o menor overhead de runtime, fine-grained reactivity sem Virtual DOM, e integração nativa com Axum para SSR. É o framework Rust frontend com melhor performance em benchmarks independentes (2025/2026).

**Controle de concorrência:** O overselling é prevenido com `SELECT ... FOR UPDATE` dentro de uma transação PostgreSQL. Duas requisições simultâneas para o último item resultam em uma transação esperando a outra terminar — a segunda recebe erro de estoque insuficiente.

**Cache:** Produtos são cacheados no Redis com TTL de 5 minutos. Qualquer operação de criação, edição ou remoção invalida a chave correspondente imediatamente.

---

## Estrutura do Repositório

```
.
├── api/                  # Backend Axum
│   ├── src/
│   │   ├── auth/
│   │   ├── catalog/
│   │   ├── cart/
│   │   ├── checkout/
│   │   └── coupons/
│   └── Cargo.toml
├── frontend/             # Frontend Leptos
│   ├── src/
│   └── Cargo.toml
├── migrations/           # SQLx migrations
├── seeds/                # Scripts de seed
├── .github/workflows/    # CI/CD GitHub Actions
├── docker-compose.yml
├── .env.example
└── README.md
```

---

*Desafio LAPES 2026 — Benjamin Yuji Suzuki*