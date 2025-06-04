# Getting Involved in Open Source

# What is Open Source and why is it important?

- Open source is a great way of learning new ways to code, I've personally
  learned an absolute tonne about Rust by simply reading the source code of the
  compiler and how people who designed it write it.
- It might be daunting at first and it can be difficult, but my experience is
  it's well worth it in the long run.
- Another example, I've read large parts of the Terraform codebase and as a
  result I'm more informed about how it works and how to use it effectively,
  which is great for something I use so frequently in my job.
- It's worth emphasising that open source is not just about code but also about
  community. The community is what makes open source so great and it's what
  makes it so much fun to be a part of.

<!-- end_slide -->

# Examples of Successful Open Source Projects

- Linux
  - The most successful open source project in the world, it's the kernel that
    powers most of the internet and most of the world's servers.
  - It's a great example of how open source can be used to create something
    that is used by millions of people every day.
  - https://www.cs.cmu.edu/~awb/linux.history.html
  - Originally felt it was a hobby project and wouldn't be that big.
- Git
  - Invented by Linus Torvalds again because he wasn't happy with the existing
    tools.
  - It's now the most popular version control system in the world. It's almost
    ubiquitous.
  - https://www.youtube.com/watch?v=4XpnKHJAok8
  - According to Linus, he hated CVS so much that the Linux kernel used
    tarballs and patches because it is "a much superior source control system
    than CVS is".
  - Little known fact, Linux was originally using BitKeeper and Linus was happy
    with this, but BitKeeper was commercial. However it allowed open source
    projects to use it for free as long as they promised not to make a competing
    product. Linus never intended to break this promise but the Linux community
    hated not using an open source tool, Linus said they were "ugly and stupid"
    but regardless they parted ways with BitKeeper and Linus wrote Git. He took
    two years out of Linux to write Git and then came back to Linux.
  - Nowadays Git is pivotal to lots of development with many CI systems being
    based around Git (like GitHub Actions) and many other tools being built
    around it like GitOps tools ArgoCD.
- Python
  - Created by Guido van Rossum in the late 80s and first released in 1991.
  - It's now one of the most popular programming languages in the world.
  - Didn't really kick off properly until the early 2000s.

<!-- end_slide -->

# More Examples of Successful Open Source Projects

- Vim
  - Created by Bram Moolenaar in 1991.
- LLVM
  - Created by Chris Lattner in 2000.
  - It's enabled a lot of new programming languages to be created.
- Docker/Kubernetes
  - Created by Solomon Hykes in 2013.
  - For a long time it was the most popular containerisation tool in the world,
    it's probably still the best known but other tools like Containerd and CRI-O
    are also very popular.
  - Kubernetes is the most popular container orchestration tool in the world
    that took over from Docker Swarm.
- Terraform
  - Created by HashiCorp in 2014.
  - It's one of the most popular infrastructure as code tool in the world.
  - Changed licence in 2023 which caused a lot of upset in the community.
  - Was forked by the community and a new project called OpenTofu was created.

<!-- end_slide -->

# Fun of licences and code of conduct

There are several options for open source licences and they all have their own
intricacies. The most popular is the MIT licence which is very permissive and
the GPL licence which is more restrictive.
<!-- pause -->

Code of conduct is a set of rules that the community agrees to follow. It is
important to have a code of conduct to ensure that the community is a safe and
welcoming place for everyone. All good, decent sized open source projects have
these and they are usually enforced by the maintainers who tend to take them
very seriously (at least you'd hope so).
<!-- pause -->

You might wonder what your rights are when you contribute to an open source
project and how that fits in with your employment. The answer is that it depends
on the licence that the project is under and it depends on how likely the
company you work for is to get upset if you contribute to a competing project.
It's always best to check with your employer but generally speaking your
employer is unlikely to mind if you contribute to anything that is not a direct
competitor to their business (if in doubt, check, ask someone).
<!-- pause -->

As an example of why licences are really important, HashiCorp recently changed
the licence of several of their projects to a more restrictive licence and
several of the community got very upset about this. They then forked the
Terraform repo and created a new project called OpenTofu and encourage people to
use their product instead. Due to the licence HashiCorp used they had every
right to fork this repo but they now have to be careful to not use any of the
new code Hashicorp has written since the licence change. This is a good example
of why licences are important and why you should always check the licence of any
project you contribute to.

<!-- end_slide -->

# How can you get involved in open source?

Find a project:
- This could be something you're already good at or something you want to learn.
- It could be something you use every day or something you've never heard of.
- It could be something that is very popular or something that is very niche.
  Very popular is harder but potentially more rewarding, very niche is easier
  and can still be very useful but is less pervasive.

<!-- end_slide -->

# What are the benefits and downsides of open source?

<!-- end_slide -->
