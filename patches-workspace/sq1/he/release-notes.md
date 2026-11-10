SQ1 EGA packages now compile dialogue, inventory names, and item descriptions
from translations.csv during the standard package build. Builds validate the
catalog and include its SHA-256 in build-report.json. CI uses a pinned resource
template and does not require original game volume archives.

No translation wording or installation targets changed in this revision.
The existing published download remains in the application catalog until a
reviewed release is published. Native Windows and in-game rendering validation
remain outstanding; this change does not mark the patch release-ready.
