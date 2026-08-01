# Gridlock

An open city at night, built from the Kenney city kits and streamed around you
as you go. You start on a sidewalk downtown, on foot, and every car on the
street is yours to take.

## The city

The city is a grid of cells, each a block of eight tiles on a side with a road
along its west and south edges, so every block has an intersection at its
corner and a crosswalk on each approach. The road tile carries the asphalt and
a curb strip, and a paved sidewalk runs between that curb and the building
faces, wide enough for the people, the lamps, the benches and the café tables,
with the crossings at the stripes. The asphalt sits a step below the sidewalk,
and a car or a person takes the height of whatever it stands on. A cell is laid out by a worker crew
from the seed and the cell's place in the grid, and the frame spawns what the
crew hands back, a bounded number of cells a frame, and takes down the cells
that fall behind you. Nothing is kept for a cell that is not near you.

The district comes from the distance to the center. Downtown carries a tower
in each block with shops around its feet, midtown carries shops and parking
lots, the works to the east carry sheds, chimneys, tanks and trucks, and the
rows beyond carry houses, fences and trees. Neon strips glow on the shop fronts
and the towers, the street lamps carry point lights, and rain falls over you
wherever you stand.

The sidewalks and the courtyards are dressed from the furniture kit, the retro
urban kit and the road kit: benches, bins, boxes, potted plants and café tables
along the shop fronts, dumpsters and dumped furniture in the alleys, garden
chairs in the yards, and pallets, bricks, beams and barriers in the works. Some
blocks are construction sites, with cones along the curb and scaffolding in the
courtyard. Every one of those props, the lamp posts included, stands in the way
of a walker and of a slow car, and a car at speed knocks it over: it tips about
its far edge, slides a little, and a lamp goes dark with its post.

## Inside

Every shop, house, shed and tower has a door on its street face. Step up to it
and the building opens: a room is built for you at that moment from the
Building Kit walls and floors, furnished for what the building is, a shop, an
office, a warehouse, a home or a lobby, and lit by its own lamp. Nothing is
kept for a room you are not in. Step back to the door and you are on the
sidewalk again, where you left.

## The people and the traffic

People walk the sidewalks from corner to corner, cross at the intersections,
stop for a while and move on, and the count in each block follows the district
and the hour. Cars drive the right lane, keep a gap to the car ahead, stop at
the light when it is red for their direction, turn at intersections along an
arc, and choose a new turn at each corner. A car that hits a person at speed
leaves them on the ground for a while, and a number bursts out over them. Each
hit inside the combo window multiplies the number, changes its color and grows
the burst, and the words beside it get less reasonable as the streak climbs.
Knocked props count toward the same streak.

## You

On foot the camera orbits over your shoulder and you walk where the camera
looks, the way hollowmere walks. The camera pulls in until it sits in your
head and the city is first person, and it pulls back out again. It stays out
of the buildings. Beside a car you can take it, and then the chase camera
follows the way the car travels, so a slide reads as a slide. The handling is
the arcade model from the hairpin app: the sideways part of the velocity
bleeds off at a grip rate that drops while the handbrake is down. Leave the
car and it goes back to being traffic.

A fly mode lifts the camera off you and lets it go anywhere at any speed, and
the city streams around the camera while it flies. The people and the cars
live only in the blocks around you, and the blocks beyond stand still.

The head-up display shows the hour, the district and, in a car, the speed. A
gamepad drives all of it as well as the keyboard and the mouse, and the menus
take the gamepad too. Every ask goes through `nightshade-input`, so the
bindings are one declaration and the file keeps what you change.

## The settings

The handling, the walk, the traffic, the crowd, the night, the cameras and the
reach of the city are one `config!` declaration. The title and the pause menu
both open the settings screen, which carries the panel built from that
declaration. A change takes effect at once, and the file keeps it between
runs.
