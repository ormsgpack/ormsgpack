project = "ormsgpack"

exclude_patterns = ["_build"]

extensions = [
    "sphinx.ext.extlinks",
    "sphinx.ext.intersphinx",
]

html_theme = "furo"
html_theme_options = {}

extlinks = {
    "issue": ("https://github.com/ormsgpack/ormsgpack/issues/%s", "#%s"),
    "pr": ("https://github.com/ormsgpack/ormsgpack/pull/%s", "#%s"),
    "user": ("https://github.com/%s", "%s"),
}

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}
