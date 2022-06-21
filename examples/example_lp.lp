a :- not b.
b :- not a.
c :- d, a.
d :- c, a.
e :- not f.
f :- not e.
c :- e.
d :- f.
d :- g.
g :- d.
h :- i.
i :- h.
i :- j.
g :- j.
k :- not j.
j :- not k.