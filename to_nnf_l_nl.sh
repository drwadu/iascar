gringo $1 --output=smodels | ./lp2normal-2.27 | ./lp2sat-1.24 > $1_nl.cnf
./c2d -in $1_nl.cnf -smooth > $1_nl_stats
gringo $1 --output=smodels | ./lp2normal-2.27 | ./lp2atomic-1.17 | ./lp2sat-1.24 > $1_l.cnf
./c2d -in $1_l.cnf -smooth > $1_l_stats