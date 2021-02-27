# Write a qaul.net Tutorial

**The qaul.net [tutorials] can be easily created. This guide explains how.**

Each tutorial is an own folder in the `/content/tutorials` folder in the [web site repository]. Please have a look at the existing tutorials to get an idea how this is done.

Here a step by step explanation on how to create a turial:

1. Create a new folder for your tutorial. Give this folder a meaningful name (without spaces or special characters in it).
2. Put all the images and files into your tutorial folder.
3. Create an `index.md` file in your tutorial folder. 
    * If you write the tutorial in another language than English, make the two letter language code part of the file name. e.g. `index.fr.md` for French.
    * Write your tutorial in text-editor. The text needs to be formated in [markdown].
    * Link your files and images in the tutorial.
    * Set the tutorial meta information in the tutorial header
        * `title` the title of your tutorial
        * `preview` the filename of an image in your tutorial folder which is used as a thumbnail image in the [tutorial overview] page.
        * `tags` define some tags that categorize the topic of your tutorial. They will be shown in the [tutorial overview] and in your tutorial.

Here an example `index.md` file

```md
---
title: 'My first Tutorial'
preview: previewImage.jpg
tags:
- MyTag
- AnotherTag
---
# This is my first Tutorial

This is the first paragraph of my tutorial with text formated in **bold** and *italic*.

Below will be shown the first image in my tutorial:

![](image.jpg)
```

4. Once you finished editing, you are ready to publish the tutorial. If you know how to use git, send us a [pull request], otherwise please send a download link to the [mailing list].



[tutorials]: https://qaul.net/tutorials/
[web site repository]: https://git.open-communication.net/qaul/website/tree/master/content/tutorials
[markdown]: https://www.markdownguide.org/getting-started
[tutorial overview]: https://qaul.net/tutorials/
[pull request]: /social/contributions.html#submitting-a-pr
[mailing list]: https://lists.sr.ht/%7Eqaul/community
