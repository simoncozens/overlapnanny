# Overlap Nanny

This is a tool for finding problems in glyph outlines caused by overlap
removal. Most existing outline checking tools consider outlines
individually and can therefore miss problems caused by the interaction
of multiple outlines. 

Overlap Nanny works by defining a measure of "path wonkiness", measuring
the wonkiness of a glyph at each named instance of the design space,
removing overlaps between the paths, and then measuring the wonkiness
again. If the wonkiness increases beyond a particular threshold value,
this is a good indicator that overlap removal is creating a visual
aberration in the glyph.

By "good indicator", we mean that manual inspection is still recommended
to confirm problems. Certain constructions of glyph (for example, such as
Q being designed as O with a rectangular tail paith) will necessarily
be found to be more wonky after overlap removal.

However, Overlap Nanny should give a list of good places to start further
inspection.

## Wonkiness

The total wonkiness of a path is the sum of the wonkiness of each node.
The "wonkiness" of a node is a measure which aims to fulfil the following
conditions:

* A colinear node (two lines at 180 degree angle) has zero wonkiness.
* A node at right angle lines has zero wonkiness.
* A sharp-angled node has a higher wonkiness than a wide-angled node.
* A shorter segment has a higher wonkiness than a longer segment. (Jagged contours are wonky.)
* A curve which is G2 continuous has zero wonkiness.
* A curve which is not G2 continuous has higher wonkiness.
* Adding a continuous point to a curve does not increase the wonkiness.
* "Line-like curves" have the same wonkiness as lines.

To measure this we define a heuristic as follows:
"the wonkiness of a node is the difference in curvature of the incoming
and outgoing segments, multiplied by the difference in angle of the
incoming and outgoing segments, divided by the sum of the lengths of the
incoming and outgoing segments."

Ideas for better heuristics are welcome.

## License

See `LICENSE` file.