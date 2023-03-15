```
# ~/.config/systemd/user/ynab.service
[Unit]
Description=Updates YNAB transactions from Akahu.
Wants=ynab.timer

[Service]
Type=oneshot
ExecStart=/home/basie/.local/share/cargo/bin/ynabber

[Install]
WantedBy=default.target

# ~/.config/systemd/user/ynab.timer
[Unit]
Description=Update YNAB transactions on a timer.
Requires=ynab.service

[Timer]
# Every 4 hours
OnCalendar=00/4:00
Persistent=true

[Install]
WantedBy=timers.target
```
