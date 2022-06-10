a :- b.
b :- a.
b :- c.
c :- b.
d :- c.
c :- d.
a :- e.
f :- g.
g :- f.
h :- not d.
d :- not h.

%%%
%Answer: 1
%d c b a
%Answer: 2
%h
%%%
%Answer: 1
%h
%Answer: 2
%h f g
%Answer: 3
%d c b a
%Answer: 4
%d c b a f g

