# ילד הינשוף — Owlboy

Own the Steam edition (app 115800). The Hebrew beta currently supports the native
macOS Steam 1.4 FNA build and Windows Steam FNA build 105-e322.
The legacy Windows XNA edition and Linux are not supported.

Install Owlboy through Steam, then use Scan or select the Owlboy installation,
Owlboy.app, or its Contents/Resources folder. Kitor resolves the same resource
folder for patching and launch. On Windows, select the folder containing Owlboy.exe
and Data (or Data itself). Python 3 must be available for the bundled delta decoder;
Windows uses the Python launcher (`py -3`). Unknown or modified executables are refused.

Close the game before installing. Choose Hebrew (the Israeli flag) in Languages.
On Windows, choose the Israeli flag marked **N** for Hebrew with nikud, or the
unmarked flag for regular Hebrew. The beta covers all 4,537 catalog entries
with Hebrew fonts and RTL rendering. Nikud comes from the upstream niqqud-beta
branch and still needs full linguistic proofreading.
A full playthrough is pending. Translation was AI-assisted and reviewed for
names, gender consistency, and game control tokens.

If the standalone Hebrew patch is already installed, uninstall it using its
own recovery journal before installing through Kitor. Kitor refuses modified
originals rather than overwriting their restoration history. Saves and settings
are outside the patch plan. Select English before uninstalling the Hebrew patch.

Source and credits: https://github.com/yhaggai/owlboy-hebrew
Owlboy and its artwork belong to D-Pad Studio. This is an unofficial fan patch.
