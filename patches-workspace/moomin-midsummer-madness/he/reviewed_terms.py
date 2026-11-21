"""Reviewed Moomin readings, preserving all text outside the matched names."""
import json
from pathlib import Path
import re

MARKS = '[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]*'
TERMS = ('השמנטפים', 'השמנטף', 'שמנטפים', 'שמנטף', 'אומללית')
GLOSSARY = json.loads(Path(__file__).with_name('niqqud-glossary.json').read_text(encoding='utf-8'))
ALTERNATIVES = '|'.join(f'(?P<t{i}>' + ''.join(re.escape(c) + MARKS for c in term) + ')'
                        for i, term in enumerate(TERMS))
PATTERN = re.compile(r'(?<![א-ת])(?P<prefix>(?:[ובכלמשה]' + MARKS + r'){0,3}?)(?:' + ALTERNATIVES + r')(?![א-ת])')


def apply_reviewed_terms(text):
    def replace(match):
        term = next(term for i, term in enumerate(TERMS) if match.group(f't{i}') is not None)
        return match.group('prefix') + GLOSSARY[term]
    return PATTERN.sub(replace, text)
