# Trumpet

## What is Trumpet?

Trumpet is a new WIP client for [Mastodon](https://mastodon.social/about/more).
Right now it is CLI-only, but a GTK+ gui using [gtk-rs](http://gtk-rs.org/) is
planned.

## Why Trumpet?

When getting started with Mastodon, I found that the only useable application
for Linux was an electron app using an indicator. Given that GNOME is phasing
out appindicators and I personally resent running more than one electron app
at once, I decided to make one myself. Since a Rust API wrapper [already
existed](https://github.com/Aaronepower/Mammut), I decided to take the
opportunity to learn the language while I worked.

## What can it do?

Right now the features Trumpet supports are:

* Multiple instances
* Multiple accounts per instance
* Text-only statuses
* Public timeline viewing
* Viewing instance information

## What's next?

Features I am working on now:

* Parsing html out of statuses viewed
* **Home timeline viewing** (Issues to work out on the Mammut end)
* Emoji support
* GTK GUI

## License informaation
```
Trumpet: A Mastodon client
Copyright (C) 2017 Christopher Davis

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
```
