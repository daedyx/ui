#!/bin/sh
# Selbsttest für .githooks/scan.sh: jedes Muster muss anschlagen, saubere Texte nicht.
# Die verbotenen Muster werden erst zur Laufzeit zusammengesetzt, damit diese Datei selbst sauber ist.
S="$(dirname "$0")/../.githooks/scan.sh"; bad=0
expect() { # expect <0|1> <beschreibung> <text>
  printf '%s\n' "$3" | sh "$S" probe >/dev/null 2>&1; got=$?
  [ "$got" -eq "$1" ] || { echo "FEHLER: $2 (erwartet $1, bekommen $got)"; bad=1; }
}
at="@"; bs='\'
expect 0 "sauberer Text" "Hallo anna.beispiel${at}example.com, Server 192.0.2.10"
expect 0 "Testdomain .test" "ben${at}post.test"
expect 0 "noreply github" "x${at}users.noreply.github.com"
expect 1 "fremde Mailadresse" "jemand${at}firma.de"
expect 1 "privater Schlüssel" "-----BEGIN RSA PRIV""ATE KEY-----"
expect 1 "GitHub-Token" "ghp_""abcdefghijklmnopqrstuvwxyz0123456789"
expect 1 "Windows-Benutzerpfad" "C:${bs}Users${bs}irgendwer${bs}x"
expect 1 "Laufwerkspfad" "D:${bs}Daten"
expect 1 "NAS-Pfad" "${bs}${bs}nas${bs}freigabe"
expect 1 "private IP" "192.168"".1.20"
expect 1 "private IP 10er" "Ziel 10.0"".0.5"
[ $bad -eq 0 ] && echo "scan-selbsttest: alles grün" || exit 1
