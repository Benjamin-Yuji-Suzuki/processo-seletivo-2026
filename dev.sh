#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
API_PORT=8099
WEB_PORT=8081

# Cores
VERDE='\033[0;32m'
AZUL='\033[0;34m'
AMARELO='\033[1;33m'
VERMELHO='\033[0;31m'
RESET='\033[0m'

info()  { echo -e "${AZUL}[INFO]${RESET} $1"; }
ok()    { echo -e "${VERDE}[OK]${RESET}   $1"; }
aviso() { echo -e "${AMARELO}[AVISO]${RESET} $1"; }
erro()  { echo -e "${VERMELHO}[ERRO]${RESET} $1"; }

cleanup() {
    echo ""
    info "Encerrando servidores..."
    kill $API_PID 2>/dev/null || true
    kill $TRUNK_PID 2>/dev/null || true
    wait $API_PID 2>/dev/null || true
    wait $TRUNK_PID 2>/dev/null || true
    ok "Tudo encerrado. Até mais!"
    exit 0
}
trap cleanup SIGINT SIGTERM

echo ""
echo -e "${AZUL}══════════════════════════════════════════${RESET}"
echo -e "${AZUL}   LAPES E-Commerce — Dev Server${RESET}"
echo -e "${AZUL}══════════════════════════════════════════${RESET}"
echo ""

# ── 1. PostgreSQL ─────────────────────────────────────
info "Verificando PostgreSQL..."
if pg_isready -q 2>/dev/null; then
    ok "PostgreSQL já está rodando"
else
    aviso "PostgreSQL não está rodando. Tentando iniciar..."
    sudo systemctl start postgresql 2>/dev/null || {
        erro "Não foi possível iniciar PostgreSQL."
        erro "Rode manualmente: sudo systemctl start postgresql"
        exit 1
    }
    sleep 2
    if pg_isready -q; then
        ok "PostgreSQL iniciado"
    else
        erro "PostgreSQL não respondeu."
        exit 1
    fi
fi

# ── 2. Redis ──────────────────────────────────────────
info "Verificando Redis..."
if redis-cli ping 2>/dev/null | grep -q PONG; then
    ok "Redis já está rodando"
else
    aviso "Redis não está rodando. Tentando iniciar..."
    sudo systemctl start redis-server 2>/dev/null || sudo systemctl start redis 2>/dev/null || {
        erro "Não foi possível iniciar Redis."
        exit 1
    }
    sleep 1
    if redis-cli ping 2>/dev/null | grep -q PONG; then
        ok "Redis iniciado"
    else
        erro "Redis não respondeu."
        exit 1
    fi
fi

# ── 3. Limpar portas ──────────────────────────────────
info "Verificando portas..."
fuser -k "${API_PORT}/tcp" 2>/dev/null || true
fuser -k "${WEB_PORT}/tcp" 2>/dev/null || true

# Aguarda portas liberarem
for port in $API_PORT $WEB_PORT; do
    for i in $(seq 1 10); do
        if ! fuser "$port/tcp" 2>/dev/null >/dev/null; then
            break
        fi
        sleep 0.5
    done
done

# ── 4. Compilar e subir API ──────────────────────────
info "Compilando e iniciando API (porta ${API_PORT})..."
cd "$ROOT_DIR"
cargo run --bin api &
API_PID=$!
sleep 2

# Espera API responder
for i in $(seq 1 30); do
    if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${API_PORT}/api/products" 2>/dev/null | grep -q 200; then
        ok "API rodando em http://localhost:${API_PORT}"
        break
    fi
    if ! kill -0 $API_PID 2>/dev/null; then
        erro "API morreu durante a inicialização."
        exit 1
    fi
    sleep 1
done

# ── 5. Compilar e subir Frontend ─────────────────────
info "Compilando e iniciando Frontend (porta ${WEB_PORT})..."
cd "$ROOT_DIR/frontend"
trunk serve --port $WEB_PORT &
TRUNK_PID=$!

# Espera frontend responder
for i in $(seq 1 60); do
    if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${WEB_PORT}/" 2>/dev/null | grep -q 200; then
        ok "Frontend rodando em http://localhost:${WEB_PORT}"
        break
    fi
    if ! kill -0 $TRUNK_PID 2>/dev/null; then
        erro "Trunk morreu durante a inicialização."
        exit 1
    fi
    sleep 1
done

# ── 6. Abrir navegador ────────────────────────────────
echo ""
echo -e "${VERDE}══════════════════════════════════════════${RESET}"
echo -e "${VERDE}   Tudo pronto!${RESET}"
echo -e "${VERDE}══════════════════════════════════════════${RESET}"
echo ""
echo -e "   Frontend:  ${AZUL}http://localhost:${WEB_PORT}${RESET}"
echo -e "   API:       ${AZUL}http://localhost:${API_PORT}${RESET}"
echo -e "   Swagger:   ${AZUL}http://localhost:${API_PORT}/swagger-ui/${RESET}"
echo ""
echo -e "   Login admin:    admin@lapes.com / admin123"
echo -e "   Login cliente:  cliente@lapes.com / customer123"
echo ""
echo -e "   ${AMARELO}Pressione Ctrl+C para parar tudo${RESET}"
echo ""

xdg-open "http://localhost:${WEB_PORT}" 2>/dev/null || true

# ── 7. Manter rodando até Ctrl+C ─────────────────────
wait
