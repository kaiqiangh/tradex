"""Check planning traceability, never product acceptance. Run with Python 3."""

import csv
import hashlib
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / "docs/implementation"


def records(name):
    with (PLAN / name).open(newline="") as source:
        return list(csv.DictReader(source))


def check():
    prd = (ROOT / "docs/TradeX_PRD_v1.0_RevC.md").read_text()
    expected = dict(re.findall(
        r"^\| ((?:FR|NFR|SEC|DATA|OPS|UX)-\d+) \| (.*?) \|", prd, re.M
    ))
    expected.update(re.findall(r"\*\*(AC-\d+)\*\*\s*\n([^\n]+)", prd))
    requirements = records("requirements.csv")
    assert len(expected) == 201, "Review the changed normative requirement inventory"
    assert len(requirements) == len(expected)
    assert {row["id"]: row["requirement"] for row in requirements} == expected

    surface_sources = [
        ("docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md", r"^### ([A-K]\d+)\. (.*)$"),
        ("docs/TradeX_Prototype_QA_Report_v1.0_RevC.md", r"^### (QA-\d+) [—–-] (.*)$"),
    ]
    expected_surfaces = {}
    for path, pattern in surface_sources:
        expected_surfaces.update(re.findall(pattern, (ROOT / path).read_text(), re.M))
    surfaces = records("surfaces.csv")
    assert len(surfaces) == len(expected_surfaces) == 82
    assert {row["id"]: row["title"] for row in surfaces} == expected_surfaces

    slices = set(re.findall(r"^\| (S\d{2}) \|", (PLAN / "README.md").read_text(), re.M))
    assert slices == {f"S{n:02}" for n in range(1, 36)}
    statuses = {"NOT_STARTED", "IMPLEMENTED_UNVERIFIED", "BLOCKED_EXTERNAL", "VERIFIED"}
    for row in requirements + surfaces:
        if row["status"] == "DEFERRED":
            assert row["id"] in {"FR-041", "FR-042", "FR-043"}
            assert row["implementation_slices"] == "DEFERRED"
            continue
        assert row["status"] in statuses, row["id"]
        owners = row["implementation_slices"].split(",")
        assert owners and set(owners) <= slices, row["id"]
        source_lines = (ROOT / row["source"]).read_text().splitlines()
        assert row["id"] in source_lines[int(row["line"]) - 1], row["id"]
        if row["status"] == "VERIFIED":
            assert row["evidence"].strip(), f"Missing evidence: {row['id']}"
    assert {r["id"] for r in requirements if r["status"] == "DEFERRED"} == {
        "FR-041", "FR-042", "FR-043"
    }

    sources = records("sources.csv")
    assert len(sources) == 23
    baseline = "9255feae39b646245acaf6db7db29fea0cb710c7"
    tracked = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", baseline, "docs"], cwd=ROOT, text=True
    ).splitlines()
    assert {r["path"] for r in sources} == set(tracked)
    for row in sources:
        assert row["review_baseline"] == baseline and row["reading_status"] == "READ"
        content = subprocess.check_output(["git", "show", f"{baseline}:{row['path']}"], cwd=ROOT)
        assert hashlib.sha256(content).hexdigest() == row["sha256"], row["path"]
        assert len(content.splitlines()) == int(row["lines"]), row["path"]
    print("Traceability OK: 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files.")
    print("This checks the plan and source inventory; it does not prove application behavior.")


if __name__ == "__main__":
    check()
