1. Umistete install.sh, alfa4 a conf.toml do stejneho adresare
2. Ujistete se, ze install.sh a alfa4 maji executable povoleni
3. Spustte skript install.sh
4. systemd service "prekladac" byl vytvoreny a lze spustit pomoci
systemd enable prekladac
systemd start prekladac

Logovaci soubor je v /etc/prekladac/log.txt
ale jeho umisteni lze zmenit

Windows
musi se zmenit log path na "log.txt" v conf.toml