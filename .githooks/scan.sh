#!/bin/sh
# Prüft einen Text (stdin) auf Geheimnisse und echte Daten (SICHERHEIT 3.2).
# Aufruf: scan.sh <anzeigename>  — Ausgabe nur Regelname und Zeilennummer, nie der Fundtext.
name="$1"
tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
cat > "$tmp"
fail=0
hit() { # hit <regel> <erweiterter-regex>
  lines="$(grep -n -E -e "$2" "$tmp" | cut -d: -f1 | head -5 | tr '\n' ' ')"
  if [ -n "$lines" ]; then echo "  $name: $1 (Zeile $lines)"; fail=1; fi
}
hit "privater Schlüssel" '-----BEGIN [A-Z ]*PRIVATE KEY-----'
hit "Minisign-Geheimschlüssel" 'untrusted comment: .*secret key'
hit "GitHub-Token" '(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{20,})'
hit "Cloud-/API-Schlüssel" '(AKIA[0-9A-Z]{16}|sk-[A-Za-z0-9_-]{20,}|xox[abprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z_-]{30,})'
hit "Windows-Benutzerpfad" '[A-Za-z]:[\/]+(Users|Benutzer)[\/]'
hit "Mac-/Linux-Benutzerpfad" '/(Users|home)/[a-z][a-z0-9_-]+/'
hit "Laufwerkspfad" '(^|[[:space:]"(=`'"'"'])[A-Za-z]:[\][A-Za-z0-9_]'
hit "Netzwerkpfad (NAS)" '[\][\][A-Za-z0-9_-]+[\]'
hit "private IP-Adresse" '(^|[^0-9.])(10\.[0-9]+\.[0-9]+\.[0-9]+|192\.168\.[0-9]+\.[0-9]+|172\.(1[6-9]|2[0-9]|3[01])\.[0-9]+\.[0-9]+)'
# Mailadressen nur aus reservierten Testbereichen (RFC 2606/6761: example.*, .test, .invalid, .example)
mails="$(grep -o -E '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$tmp" \
  | grep -v -i -E '@([a-z0-9-]+\.)*(example\.(com|org|net)|[a-z0-9-]+\.test|[a-z0-9-]+\.invalid|[a-z0-9-]+\.example|users\.noreply\.github\.com)$' | head -1)"
if [ -n "$mails" ]; then echo "  $name: Mailadresse außerhalb example.com/.test/.invalid"; fail=1; fi
# Lokale Wortliste (echte Namen usw.), liegt nie im Repo
wl="${APPDATA:-}/Daedyx/verboten.txt"
if [ -f "$wl" ] && grep -q -i -F -f "$wl" "$tmp"; then echo "  $name: Wort aus der lokalen Verbotsliste"; fail=1; fi
exit $fail
