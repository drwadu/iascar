LP=$1

GRINGO=gringo
LP2NORMAL=bins/lp2normal-1.14
LPSHIFT=bins/lpshift-1.2
LP2ATOMIC=bins/lp2atomic-1.17
LP2SAT=bins/lp2sat-1.24
C2D=bins/c2d

$GRINGO --output=smodels $LP | $LPSHIFT | $LP2NORMAL | $LP2ATOMIC | $LP2SAT > $1.dlp.as.cnf
$GRINGO --output=smodels $LP | $LPSHIFT | $LP2NORMAL | $LP2SAT > $1.dlp.sm.cnf

$C2D -in $1.dlp.as.cnf -smooth
$C2D -in $1.dlp.sm.cnf -smooth
