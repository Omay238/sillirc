# sillirc
###### An IRC type thing for the silliest of folks.

<img src="sillirc.svg" width="128" alt="sillirc logo" />

For Hack Club Siege Week 3!

## Theme
The theme for this week was "signal," and my first idea was a p2p messaging platform, however, after thinking and trying
for an hour, I decided that was far too hard. Because of that, I went for something that would still be pretty tricky,
but much less so. I've wanted to make a chat app of sorts for quite a while, however have failed every time, but at
least so far, I'm making good progress.

## Project Structure
There are four main projects in this repo. The first (and main) is `sillirc`, which is the frontend, written with egui,
based on the eframe-template repository. The second is `sillirc-mini` which is a semi-functional tui version of
`sillirc`, which was made just because it would be easier to test the server. Speaking of the server, the third project
is`sillirc-server`, which is the backend. Currently, the server defaults to the instance I have hosted on Hack Club's
very generously provided "Nest" service. The fourth and final project is `sillirc-lib`, which is the library stuff I
have written for making networking and other stuff easier.

## Running
There are releases available on GitHub over on the right, but if you want the latest version, you've got two options.
The first, and better, is to go to [nightly.link](https://nightly.link/Omay238/sillirc/workflows/rust/main?preview), and
get it there. Then, if you plan on development, you can `git clone` the repository, navigate into it, and with cargo
installed, just run `cargo run --release --bin sillirc` for the GUI.