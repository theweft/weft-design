# SPDX-FileCopyrightText: 2026 The weft authors
# SPDX-License-Identifier: MIT
"""StrictDoc configuration: system requirements in requirements/."""

from strictdoc.core.project_config import ProjectConfig


def create_config() -> ProjectConfig:
    return ProjectConfig(
        project_title="weft system requirements",
        include_doc_paths=["requirements/**"],
    )
