# sillirc
###### An IRC type thing for the silliest of folks.

For Hack Club Siege Week 3!

## Theme
The theme for this week was "signal," and my
first idea was a p2p messaging platform,
however, after thinking and trying for an
hour, I decided that was far too hard.
Because of that, I went for something that
would still be pretty tricky, but much less
so. I've wanted to make a chat app of sorts
for quite a while, however have failed
every time, but at least so far, I'm making
good progress.

## Project Structure
There are three main projects in this repo.
The first (and main) is `sillirc`, which is
the frontend, written with egui, based on
the eframe-template repository. The second
is `sillirc-mini` which is a
semi-functional tui version of `sillirc`,
which was made just because it would be
easier to test the server. Speaking of
the server, the third project is
`sillirc-server`, which is the backend.
Currently, the server defaults to the
instance I have hosted on Hack Club's very
generously provided "Nest" service.

## Running
I should have releases available in GitHub,
but if not, you can pretty easily build
from source.