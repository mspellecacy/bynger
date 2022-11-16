# Bynger
Bynger's goal is to let users create their own TV & Movie (content) schedules agnostic of streaming services.
Schedule and Create their own "Must See TV" lineups or Sunday night horror movie marathon.


... and hopefully, eventually, share them with their friends. 

## How to set up and run:
Bynger uses Trunk to package and run, you can easily [install it with Cargo](https://trunkrs.dev/#getting-started).

Once you have Trunk you can then run ```trunk serve``` from cli at the project root.
Assuming everything compiled and launched correctly you can now go to [Bynger's config](http://localhost:8080/config) and then give it your [TMDB API Key](https://developers.themoviedb.org/3/getting-started/introduction). 

Currently, there is no standalone release to launch Bynger, perhaps in the future? (make a PR)

## How to use:
Once Bynger is configured with a working TMDB API Key, you can start searching for content you want to schedule: 
![Basic Demo](https://i.imgur.com/UEfz6Wv.gif)


## Want to contribute?
Please feel free to open a ticket, create a fork, or make a pull request. 
I'm always open to suggestions and recommendations for new features.

#### Notes
This project is a rewrite of an App I originally wrote in Angular/TypeScript as a means to learn Rust on the Web.

Cheers!
