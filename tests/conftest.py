import os
import signal
import ssl
import subprocess
import time

import pytest
import httpx

TEST_DIR = os.path.dirname(os.path.abspath(__file__))
BOLT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
SERVERS_DIR = os.path.join(TEST_DIR, "servers")

_MODULE_PORTS = {
    "test_http_methods": 8801,
    "test_routing": 8802,
    "test_middleware": 8803,
    "test_response": 8804,
    "test_cookies_sessions": 8805,
    "test_auth": 8806,
    "test_caching": 8807,
    "test_websocket": 8808,
    "test_sse": 8809,
    "test_uploads": 8810,
    "test_utilities": 8811,
    "test_security": 8812,
    "test_docs_homepage": 8813,
    "test_static_files": 8814,
    "test_cors_compression": 8815,
    "test_logging": 8816,
    "test_advanced_ws": 8817,
    "test_server_config": 8818,
    "test_environment": 8819,
    "test_tls": 8820,
    "test_security_edge": 8821,
    "test_ws_limits": 8822,
    "test_sse_limits": 8823,
    "test_limits": 8824,
    "test_panics": 8825,
    "test_error_paths": 8826,
    "test_wildcard_routes": 8827,
    "test_json_booleans": 8828,
    "test_openapi_wildcards": 8829,
}


def _port_for_module(module_name: str) -> int:
    short = module_name.split(".")[-1]
    return _MODULE_PORTS.get(short, 8899)


@pytest.fixture(scope="session")
def ring_cmd():
    return os.environ.get("RING_CMD", "ring")


@pytest.fixture
def bolt_server(ring_cmd, request):
    module_name = request.module.__name__
    short_name = module_name.split(".")[-1]
    server_name = short_name.removeprefix("test_")
    server_file = os.path.join(SERVERS_DIR, f"{server_name}.ring")
    if not os.path.exists(server_file):
        pytest.skip(f"Server file not found: {server_file}")

    port = _port_for_module(module_name)
    env = os.environ.copy()
    env["BOLT_TEST_PORT"] = str(port)
    env["BOLT_TEST_DIR"] = TEST_DIR

    proc = subprocess.Popen(
        [ring_cmd, server_file],
        cwd=BOLT_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
        preexec_fn=os.setsid,
    )

    base_url = f"http://127.0.0.1:{port}"
    use_tls = "tls" in short_name
    if use_tls:
        base_url = f"https://127.0.0.1:{port}"

    ctx = None
    if use_tls:
        ctx = ssl.create_default_context()
        ctx.load_verify_locations(os.path.join(TEST_DIR, "certs", "cert.pem"))
        ctx.check_hostname = False

    started = False
    for _ in range(40):
        try:
            r = httpx.get(f"{base_url}/health", timeout=1, verify=ctx or True)
            if r.status_code == 200:
                started = True
                break
        except (httpx.ConnectError, httpx.ReadTimeout):
            pass
        if proc.poll() is not None:
            out = proc.stdout.read().decode(errors="replace")
            err = proc.stderr.read().decode(errors="replace")
            pytest.fail(f"Server exited early:\nSTDOUT:\n{out}\nSTDERR:\n{err}")
        time.sleep(0.25)

    if not started:
        os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        pytest.fail(f"Server at {base_url} never became ready")

    yield base_url

    os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        proc.wait()


@pytest.fixture
def client(bolt_server):
    with httpx.Client(base_url=bolt_server, follow_redirects=False) as c:
        yield c


def unwrap(data):
    """Bolt's json() wraps responses in {"Ok": ...}. Unwrap for assertions."""
    if isinstance(data, dict) and "Ok" in data and len(data) == 1:
        return data["Ok"]
    return data
