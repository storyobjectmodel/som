# Introduction and principles

*Jon Roberts, for the authors. September 2026.*

## What this is

The Story Object Model is a small, open standard for one thing: sharing what a story is,
and what is true about it right now, between every system that touches it. It is a set of
JSON message families published on a bus. Any system can say what it knows about a story;
any other system can act on that without a point-to-point integration and without asking
anyone's permission. That is the whole of it. The rest of this repository is the detail.

## Why it did not exist

Every wave of production technology has really been a wave of integration. MOS let
newsroom systems drive devices and changed what a gallery could be. Automation stitched
control rooms into coherent machines. APIs, where we have them, opened systems to each
other. Three decades of that work, and it is worth being honest about its state: behind
most facilities sits a web of point-to-point connections and bespoke middleware that holds
because somebody maintains it, and some of our most important systems are joined by
nothing more than the video signal passing between them.

All of that effort moves the same four things: media, cues, commands and technical
metadata. Media had a model, so media moved. Control had a model, so control moved. The
story never had a model, so the story never moved. Nobody failed to build the editorial
layer; nobody attempted it. It was invisible, because people carried it, and people are
very good at carrying it, right up to the moment they cannot.

That moment has arrived from two directions at once. The volume of a modern newsroom on
a big night is beyond anything a person at every joint can carry. And the systems that are
now joining the production stack, the models and the agents, are only as good as the
editorial state you can hand them. An agent without context guesses, confidently and
fluently and sometimes wrongly. The field calls the answer context engineering. In
production, the context is the story.

## The idea

Tools stop talking to each other. They talk to a shared, live description of the story.

Everything in SOM sits in one of three layers. The **Story** is the happening, the
reportable event, held as a context that a publisher owns and asserts. An **Asset** is a
discrete piece of media or editorial work, classified by where it sits in the evidential
chain: primary, secondary or tertiary. Assets exist whether or not anything is ever
published from them. A **Telling** is the moment an asset meets an audience through a
destination, with its own record of when exposure started and ended.

Those three nouns were not invented. They were transcribed, in sessions we called Story
Archaeology, from the way working newsrooms already describe what they do. The model is
small because the vocabulary it records is small; a newsroom has been running on it for
a century without writing it down.

Around the three layers sit the things a story needs in order to travel: lifecycle,
priority, the editorial gates that say what may and may not go out, the compliance
position, where each piece of material came from and how much it is trusted, and how one
story relates to another. Every one of those is state the publisher asserts. None of it is
inferred by the bus.

## What SOM is not

SOM is not a rundown. The rundown is one view of a story; SOM carries the story the
rundown is a view of. SOM is not a media store. Media never travels the bus; a story
points at frames held elsewhere, in a media store such as TAMS, by a source reference and
a time range, and the binding between story and media happens at the editorial act, never
at ingest. SOM is not an orchestrator. Nothing is in charge. There is no central
application deciding what happens next; each system reads the story and decides for
itself, and when the bus is down every tool keeps working and reconciles when it returns.

And SOM is not the judgement. What a newsroom does with a story, which flags it raises and
which actions it holds, is house policy. SOM carries the context; Skills carry the
knowledge. A skill declares and displays; the tool that runs it decides. The framework is
open and shared; your workflow is yours.

## The principles

Six working principles shaped every decision in this specification, and they are the
test any future change has to pass.

**Single-story scope.** The standard coordinates within one story. A story may declare
its relationships to other stories, and those travel with it as references. The standard's
scope stops there.

**Messages, not organisations.** SOM describes what travels on the bus, not how a
publisher organises itself internally. It has no opinion about your desks, your roles or
your tools.

**Publisher-asserted state.** A story context says what its owner says is true. It never
carries a guess derived across sources. The story is minted once, by whoever owns it, and
everyone else refers to it by that identity.

**Coordination, not consumption.** SOM carries what other systems need in order to act
together. It does not carry the content itself, and it is not a delivery format.

**Default no, justify yes.** Nothing goes into the schema because it might be useful. A
field earns its place by blocking a real integration without it. Version 1.0 withdrew
three items that had been proposed but never ratified, for exactly this reason; they are
recorded in the open register and can return the moment they are settled.

**Vocabulary growth governed.** The core vocabulary is small and stable. Extension is
expected and has a defined shape, so that a vendor's terms and a newsroom's terms can
share one message without either being second class.

## Published, not finished

1.0 is a commitment to stability, not a claim of completeness. The contract in `schema/`
is stable; what the working group knows it has not yet settled is written down in the
open register rather than left for you to discover. The standard is open and unowned.
Implement it. Pressure test it. Argue with it. Then tell us what broke.

Context is infrastructure. It is time we built it like infrastructure.
