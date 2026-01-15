import nox

nox.options.default_venv_backend = "uv"
nox.options.reuse_existing_virtualenvs = True

PYPROJECT = nox.project.load_toml("pyproject.toml")
PYTHON_VERSIONS = nox.project.python_versions(PYPROJECT)


@nox.session(python="3.13")
def lint(session: nox.Session) -> None:
    session.run_install(
        "uv",
        "sync",
        "--frozen",
        f"--python={session.virtualenv.location}",
        env={"UV_PROJECT_ENVIRONMENT": session.virtualenv.location},
    )
    session.run("ruff", "format", "--check", ".")
    session.run("ruff", "check", ".")
    session.run("mypy", "python", "tests")
    session.run("cargo", "fmt", "--check", external=True)
    session.run("cargo", "clippy", "--", "-D", "warnings", external=True)


@nox.session(
    python=[
        *PYTHON_VERSIONS,
        "3.14t",
        "graalpy-3.11",
        "pypy-3.11",
    ]
)
def test(session: nox.Session) -> None:
    session.run_install(
        "uv",
        "sync",
        "--frozen",
        f"--python={session.virtualenv.location}",
        env={"UV_PROJECT_ENVIRONMENT": session.virtualenv.location},
    )
    session.run("pytest")


@nox.session(python="3.13")
def docs(session: nox.Session) -> None:
    session.run_install(
        "uv",
        "sync",
        "--frozen",
        "--only-group=docs",
        f"--python={session.virtualenv.location}",
        env={"UV_PROJECT_ENVIRONMENT": session.virtualenv.location},
    )
    session.run("sphinx-build", "-b", "html", "-W", "docs", "docs/html")
