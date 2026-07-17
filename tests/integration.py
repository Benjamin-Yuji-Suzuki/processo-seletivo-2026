#!/usr/bin/env python3
"""
Testes de integração — LAPES E-commerce API

Uso:
  python3 tests/integration.py

Requer API rodando em http://localhost:8099
"""
import datetime, json, os, sys, threading, time, uuid
import urllib.error, urllib.request
from dataclasses import dataclass, field
from typing import Any, Optional

API = os.environ.get("API_URL", "http://localhost:8099/api")
PASS = "✅"
FAIL = "❌"

# ── HTTP Client ──────────────────────────────────────────────────────────

@dataclass
class Client:
    token: str = ""
    admin_token: str = ""

    def req(self, method: str, path: str, data: dict = None,
            token: str = None, idempotency_key: str = None,
            status: int = 200) -> dict:
        url = f"{API}{path}"
        body = json.dumps(data).encode() if data is not None else None
        headers = {"Content-Type": "application/json"}
        t = token or self.token
        if t:
            headers["Authorization"] = f"Bearer {t}"
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        r = urllib.request.Request(url, data=body, headers=headers, method=method)
        try:
            resp = urllib.request.urlopen(r)
        except urllib.error.HTTPError as e:
            body = e.read().decode()
            if e.code == status:
                return json.loads(body) if body else {}
            raise AssertionError(f"HTTP {e.code} (expected {status}): {body[:200]}")
        assert resp.status == status, f"Expected HTTP {status}, got {resp.status}"
        ct = resp.headers.get("Content-Type", "")
        if "json" in ct:
            return json.loads(resp.read())
        return {"_text": resp.read().decode()}

    def login(self, email: str, password: str) -> str:
        r = self.req("POST", "/auth/login", {"email": email, "password": password})
        return r["token"]


# ── Test Runner ──────────────────────────────────────────────────────────

passed = 0
failed = 0

def check(name: str, cond: bool, detail: str = ""):
    global passed, failed
    if cond:
        passed += 1
        print(f"  {PASS} {name}")
    else:
        failed += 1
        msg = f"  {FAIL} {name}" + (f" — {detail}" if detail else "")
        print(msg)

# ── Tests ────────────────────────────────────────────────────────────────

def run_all(c: Client):
    global passed, failed

    # ── Login admin once ───────────────────────────────────────────────
    c.admin_token = c.login("admin@lapes.com", "admin123")
    check("Login admin", bool(c.admin_token))

    # ── Health ─────────────────────────────────────────────────────────
    print(f"\n── Health Check ──")
    r = c.req("GET", "/health")
    check("Status healthy", r["status"] == "healthy")
    check("Database up", r["checks"]["database"]["status"] == "up")
    check("Version presente", "version" in r)

    r = c.req("GET", "/metrics")
    check("Métricas Prometheus", "axum_http_requests_total" in str(r))
    check("Contagem de requests", "axum_http_requests_duration_seconds" in str(r))

    # ── Auth ───────────────────────────────────────────────────────────
    print(f"\n── Autenticação ──")
    ts = str(int(time.time()))
    test_email = f"test_{ts}@lapes.com"

    r = c.req("POST", "/auth/register",
              {"name": "Test User", "email": test_email,
               "password": "123456"}, status=201)
    check("Registrar usuário", "token" in r)

    r = c.req("POST", "/auth/register",
              {"name": "Dup", "email": test_email,
               "password": "123456"}, status=409)
    check("Email duplicado rejeitado", r.get("error", "") != "")

    c.token = c.login(test_email, "123456")
    check("Login OK", bool(c.token))

    c.req("POST", "/auth/login",
          {"email": test_email, "password": "wrong"}, status=401)
    check("Senha errada rejeitada", True)

    r = c.req("GET", "/auth/me", token=c.token)
    check("GET /me funciona", r.get("email") == test_email)

    # ── Catalog ────────────────────────────────────────────────────────
    print(f"\n── Catálogo ──")
    r = c.req("GET", "/products")
    check("Listar produtos público", r["total"] > 0)

    r = c.req("GET", "/products?search=Fone")
    check("Buscar por nome", r["total"] > 0)

    r = c.req("GET", "/products?page=1&per_page=5")
    check("Paginação", len(r["products"]) <= 5)

    r = c.req("POST", "/products",
              {"name": "Teste Delete", "description": "Será deletado",
               "price": 10.0, "category": "Teste", "image_url": "", "stock": 5},
              token=c.admin_token, status=201)
    prod_id = r["id"]
    check("Criar produto (admin)", prod_id is not None)

    c.req("POST", "/products",
          {"name": "Hack", "description": "x", "price": 1,
           "category": "x", "image_url": "", "stock": 1}, status=403)
    check("Criar sem admin bloqueado", True)

    r = c.req("GET", f"/products/{prod_id}")
    check("GET produto por ID", r["name"] == "Teste Delete")

    # Soft delete
    c.req("DELETE", f"/products/{prod_id}", token=c.admin_token, status=204)
    check("Soft delete (HTTP 204)", True)

    c.req("GET", f"/products/{prod_id}", status=404)
    check("Produto deletado some", True)

    # ── Cart ───────────────────────────────────────────────────────────
    print(f"\n── Carrinho ──")

    # Create a dedicated product with plenty of stock for cart/checkout tests
    r = c.req("POST", "/products",
              {"name": "Produto Teste", "description": "Item para testes",
               "price": 50.0, "category": "Teste", "image_url": "", "stock": 999},
              token=c.admin_token, status=201)
    pid = r["id"]
    check("Criar produto teste (admin)", pid is not None)

    r = c.req("POST", "/cart", {"product_id": pid, "quantity": 2}, token=c.token)
    check("Adicionar ao carrinho", r.get("quantity") == 2)
    cart_prod_id = r.get("product_id")

    r = c.req("GET", "/cart", token=c.token)
    check("Listar carrinho", len(r["items"]) > 0 and r["total"] > 0)

    if cart_prod_id:
        r = c.req("PUT", f"/cart/{cart_prod_id}", {"quantity": 1}, token=c.token)
        check("Atualizar quantidade", r.get("quantity") == 1)

    c.req("GET", "/cart", token=c.token)
    r = c.req("POST", "/cart",
              {"product_id": "00000000-0000-0000-0000-000000000000", "quantity": 1},
              token=c.token, status=404)
    check("Produto inexistente rejeitado", True)

    # ── Checkout ───────────────────────────────────────────────────────
    print(f"\n── Checkout ──")
    # Add item and checkout
    r = c.req("POST", "/cart", {"product_id": pid, "quantity": 1}, token=c.token)
    r = c.req("POST", "/checkout", {}, token=c.token)
    check("Checkout cria pedido", r.get("id") is not None)
    check("Status paid", r["status"] == "paid")
    check("Total > 0", r["final_total"] > 0)
    last_order = r["id"]

    # Empty cart checkout
    c.req("POST", "/checkout", {}, token=c.token, status=400)
    check("Checkout carrinho vazio rejeitado", True)

    # ── Idempotency ────────────────────────────────────────────────────
    print(f"\n── Idempotência ──")
    key = str(uuid.uuid4())
    c.req("POST", "/cart", {"product_id": pid, "quantity": 1}, token=c.token)
    r1 = c.req("POST", "/checkout", {}, token=c.token, idempotency_key=key)
    # Add another item to cart — 2nd call with same key should return cached
    c.req("POST", "/cart", {"product_id": pid, "quantity": 1}, token=c.token)
    r2 = c.req("POST", "/checkout", {}, token=c.token, idempotency_key=key)
    check("Idempotência: mesmo order_id", r1["id"] == r2["id"])
    check("Idempotência: mesmo total", r1["final_total"] == r2["final_total"])

    # ── Orders ─────────────────────────────────────────────────────────
    print(f"\n── Pedidos ──")
    r = c.req("GET", "/orders", token=c.token)
    check("Listar meus pedidos", len(r) > 0)

    r = c.req("GET", f"/orders/{last_order}", token=c.token)
    check("Detalhar pedido", r["id"] == last_order)

    r = c.req("PUT", f"/orders/{last_order}/cancel", token=c.token)
    check("Cancelar pedido", r.get("message") == "Pedido cancelado com sucesso")

    c.req("PUT", f"/orders/{last_order}/cancel", token=c.token, status=400)
    check("Cancelar já cancelado rejeitado", True)

    # Admin: list all orders
    r = c.req("GET", "/orders/all", token=c.admin_token)
    check("Admin lista todos pedidos", len(r) > 0)

    c.req("GET", "/orders/all", token=c.token, status=403)
    check("Customer não lista todos", True)

    # ── Coupons ────────────────────────────────────────────────────────
    print(f"\n── Cupons ──")
    exp = (datetime.datetime.now(datetime.timezone.utc) +
           datetime.timedelta(days=30)).isoformat()
    ccode = f"T10_{int(time.time())}"
    ccode2 = f"FIX_{int(time.time())}"
    r = c.req("POST", "/coupons",
              {"code": ccode, "discount_type": "percentage",
               "discount_value": 10.0, "expires_at": exp},
              token=c.admin_token)
    check("Criar cupom percentual", r["code"] == ccode)

    r = c.req("POST", "/coupons",
              {"code": ccode2, "discount_type": "fixed",
               "discount_value": 20.0, "expires_at": exp},
              token=c.admin_token)
    check("Criar cupom valor fixo", r["discount_type"] == "fixed")

    c.req("POST", "/coupons",
          {"code": "NOPE", "discount_type": "percentage",
           "discount_value": 10, "expires_at": exp}, status=403)
    check("Criar cupom sem admin bloqueado", True)

    r = c.req("POST", "/coupons/validate",
              {"code": ccode, "total": 100.0})
    check("Validar cupom válido", r.get("valid") is True and r["discount"] == 10.0)

    # Expired coupon
    past = (datetime.datetime.now(datetime.timezone.utc) -
            datetime.timedelta(days=1)).isoformat()
    exp_code = f"EXP_{int(time.time())}"
    c.req("POST", "/coupons",
          {"code": exp_code, "discount_type": "percentage",
           "discount_value": 50, "expires_at": past},
          token=c.admin_token)
    r = c.req("POST", "/coupons/validate",
              {"code": exp_code, "total": 100.0})
    check("Cupom expirado rejeitado", r.get("valid") is False)

    # Min order value - create a coupon with minimum
    min_code = f"MIN_{int(time.time())}"
    c.req("POST", "/coupons",
          {"code": min_code, "discount_type": "fixed",
           "discount_value": 10, "min_order_value": 100.0, "expires_at": exp},
          token=c.admin_token)
    r = c.req("POST", "/coupons/validate",
              {"code": min_code, "total": 5.0})
    check("Abaixo do mínimo", r.get("valid") is False)

    # Checkout with coupon
    r = c.req("POST", "/cart", {"product_id": pid, "quantity": 1}, token=c.token)
    r = c.req("POST", "/checkout", {"coupon_code": ccode}, token=c.token)
    check("Checkout com cupom", r.get("discount", 0) > 0)
    check("Final total < total", r["final_total"] < r["total"])

    # ── Concurrency ────────────────────────────────────────────────────
    print(f"\n── Concorrência ──")
    r = c.req("POST", "/products",
              {"name": "Item Único", "description": "Só 1",
               "price": 5.0, "category": "Concorrência",
               "image_url": "", "stock": 1},
              token=c.admin_token, status=201)
    singleton_id = r["id"]

    results = []
    def attempt_checkout(label: str):
        try:
            uid = str(uuid.uuid4())[:8]
            email = f"conc_{uid}@test.com"
            c.req("POST", "/auth/register",
                  {"name": f"Conc{label}", "email": email, "password": "123456"},
                  status=201)
            tok = c.login(email, "123456")
            c.req("POST", "/cart",
                  {"product_id": singleton_id, "quantity": 1}, token=tok)
            resp = c.req("POST", "/checkout", {}, token=tok)
            results.append(("ok", resp["id"]))
        except AssertionError as e:
            results.append(("blocked", str(e)[:80]))
        except Exception as e:
            results.append(("error", str(e)[:80]))

    t1 = threading.Thread(target=attempt_checkout, args=("A",))
    t2 = threading.Thread(target=attempt_checkout, args=("B",))
    t1.start(); t2.start()
    t1.join(); t2.join()

    successes = [r for r in results if r[0] == "ok"]
    blocked = [r for r in results if r[0] == "blocked"]
    check("Concorrência: só 1 compra bem-sucedida",
          len(successes) == 1,
          f"Sucessos: {len(successes)}, Bloqueados: {len(blocked)}")

    # ── Soft-deleted no cart ───────────────────────────────────────────
    print(f"\n── Soft Delete no Carrinho ──")
    r = c.req("POST", "/products",
              {"name": "Já Era", "description": "Vai sumir",
               "price": 1.0, "category": "Teste",
               "image_url": "", "stock": 5},
              token=c.admin_token, status=201)
    del_id = r["id"]
    c.req("DELETE", f"/products/{del_id}", token=c.admin_token, status=204)

    uid2 = str(uuid.uuid4())[:8]
    email2 = f"sdel_{uid2}@test.com"
    c.req("POST", "/auth/register",
          {"name": "Sdel", "email": email2, "password": "123456"}, status=201)
    tok2 = c.login(email2, "123456")
    c.req("POST", "/cart",
          {"product_id": del_id, "quantity": 1}, token=tok2, status=404)
    check("Adicionar produto deletado rejeitado", True)


# ── Main ─────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    c = Client()
    try:
        run_all(c)
    except Exception as e:
        print(f"\n{FAIL} Test suite crash: {e}")
        import traceback
        traceback.print_exc()

    total = passed + failed
    print(f"\n── Resumo ──")
    print(f"  {PASS} {passed} passaram")
    print(f"  {FAIL} {failed} falharam" if failed else f"  {PASS} todas passaram!")
    sys.exit(0 if failed == 0 else 1)
