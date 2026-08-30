# PDF Link Map demo

## CLI sample

Run `pdf-link-map --demo` after installing the binary. It creates a two-page
sample PDF and heading manifest in a uniquely named temporary directory,
audits them with the same parser used for a real PDF, and prints the standalone
HTML report location. The sample includes one valid internal link, one broken
anchor, and one external URI that is recorded but never opened. The command
does not read project files and does not send a request.

Use `pdf-link-map --demo --json` to write only the sample audit JSON to stdout.
The generated sample is disposable; rerun the command for a fresh copy.

## Browser sample

Open `/?demo=1#demo` or select **Try it with sample data** on the first screen.
It displays bundled specimen results only. Demo mode has no storage namespace
because the static page does not save any visitor data. The persistent banner
states this, offers **Reset demo**, and takes **Start for real** visitors to
the local CLI install command.
