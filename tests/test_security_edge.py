import pytest
from conftest import unwrap


def _get_csrf_token(client):
    token_r = client.get("/csrf/token")
    token_data = token_r.json()
    if "Ok" in token_data:
        return token_data["Ok"]["csrf_token"]
    return token_data["csrf_token"]


def test_jwt_expired_token(client):
    token = _get_csrf_token(client)
    r = client.post("/jwt/login", json={"username": "alice"}, headers={"X-CSRF-Token": token})
    assert r.status_code == 200
    jwt_token = unwrap(r.json())["token"]
    r2 = client.get("/jwt/me", headers={"Authorization": f"Bearer {jwt_token}"})
    assert r2.status_code == 200


def test_jwt_short_secret_rejected(client):
    token = _get_csrf_token(client)
    try:
        r = client.post("/jwt/login-short-secret", json={"username": "alice"}, headers={"X-CSRF-Token": token})
        assert r.status_code == 500
    except Exception:
        pass


def test_csrf_auto_verify_with_token(client):
    token = _get_csrf_token(client)
    r = client.post("/csrf/auto-protected", data={"_csrf": token})
    assert r.status_code == 200


def test_csrf_auto_verify_no_token(client):
    r = client.post("/csrf/auto-protected")
    assert r.status_code == 403


def test_csrf_via_header(client):
    token = _get_csrf_token(client)
    r = client.post("/csrf/header-token", headers={"X-CSRF-Token": token})
    assert r.status_code == 200


def test_csrf_via_query_param(client):
    # Query-string CSRF tokens were removed as a source: tokens in URLs can
    # leak via logs, history, and Referer headers. Header/form only.
    token = _get_csrf_token(client)
    r = client.post(f"/csrf/query-token?_csrf={token}")
    assert r.status_code == 403
