LP=$1

GRINGO=gringo
LP2NORMAL=bins/lp2normal-2.27
LP2ATOMIC=bins/lp2atomic-1.17
LP2SAT=bins/lp2sat-1.24
C2D=bins/c2d

$GRINGO --output=smodels $LP | $LP2NORMAL | $LP2ATOMIC | $LP2SAT > $1.as.cnf
$GRINGO --output=smodels $LP | $LP2NORMAL | $LP2SAT > $1.sm.cnf

$C2D -in $1.as.cnf -smooth
$C2D -in $1.sm.cnf -smooth
