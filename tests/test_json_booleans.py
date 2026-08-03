import pytest
from conftest import unwrap


def test_flags_encode_real_booleans(client):
    r = client.get("/flags")
    assert r.status_code == 200
    data = unwrap(r.json())
    assert data["active"] is True
    assert data["disabled"] is False
    assert data["count"] == 1
    assert not isinstance(data["count"], bool)


def test_decode_predicates(client):
    r = client.get("/decode")
    assert r.status_code == 200
    data = unwrap(r.json())
    assert data["enabled"] == 1
    assert data["muted"] == 1
    assert data["enabledisfalse"] == 0
    assert data["mutedistrue"] == 0
    assert data["asbool"] == 1
    assert data["asboolfalse"] == 0
    assert data["score"] == 1


def test_roundtrip_preserves_booleans(client):
    r = client.get("/roundtrip")
    assert r.status_code == 200
    data = unwrap(r.json())
    assert '"active":true' in data["restored"]
    assert '"disabled":false' in data["restored"]
    assert '"count":1' in data["restored"]
