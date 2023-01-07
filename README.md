# Local Reverse Image Search

![](LRIS_demo.gif)

**Description**:  This program was originally created to help my dad determine whether a filmstock image had alread been scanned. It detects Akaze keypoints and their descriptors in images then finds matches between those in some query image and those in a library of search images. Images with a significantly-higher number of matched points are considered overall matches for the query image. 

But I also view it as a fun sandbox to experiment with Rust and new features. Feel free to submit a pull request for a new feature you think would be cool. Extending the program's functionality to other file types is one that has been brainstormed a bit already.

## Dependencies

No dependencies outside of those specified in the Cargo.toml file should be needed.

## Installation

1. Install Rust [(guide here)](https://www.rust-lang.org/tools/install)
2. Clone this repository ```git clone https://github.com/benhamlen/local-reverse-image-search.git```

## Configuration

```config.toml``` is the main configuration document for this program.

The most important configuration is the search directory paths.

## Usage

1. Run the program with ```cargo run --release```
2. Select a query image

Show users how to use the software.
Be specific.
Use appropriate formatting when showing code snippets.

## How to test the software

If the software includes automated tests, detail how to run those tests.

## Known issues

Document any known significant shortcomings with the software.

**Example**

If you have questions, concerns, bug reports, etc, please file an issue in this repository's Issue Tracker.

## Getting involved

This section should detail why people should get involved and describe key areas you are
currently focusing on; e.g., trying to get feedback on features, fixing certain bugs, building
important pieces, etc.

General instructions on _how_ to contribute should be stated with a link to [CONTRIBUTING](CONTRIBUTING.md).


----

## Open source licensing info
1. [TERMS](TERMS.md)
2. [LICENSE](LICENSE)
3. [CFPB Source Code Policy](https://github.com/cfpb/source-code-policy/)


----

## Credits and references

1. Projects that inspired you
2. Related projects
3. Books, papers, talks, or other sources that have meaningful impact or influence on this project
