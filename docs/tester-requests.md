# Tester requests

A chat between testers and the coding assistant, kept in a file. The tester writes what they want in a box, the assistant replies in its own box, the tester answers, and so on, the same as a normal chat. When the tester is happy, they give the go-ahead and the assistant starts coding. When they're satisfied with the result, they close the thread.

Each thread has a title and is separated from the next one by a line. Finished threads move to `tester-requests-archive.md`.

## Contents


---

## For the assistant: read this first

  

Before replying to anything here, go read `design-philosophy-instructions.md` in this folder and follow it, along with the docs it points to. Those are the project's rules, and they win if anything in this file seems to disagree with them. Two things work differently here:

- The tester's `Go ahead` in a thread is the go-ahead to start coding (the instructions' "don't write code until I give the go-ahead" is satisfied by it).

- Replies follow the chat format below.

  

**Where threads live.** Each thread is its own file: `testers/<thread>/<thread>.md` (the thread's title in lowercase with hyphens, in the thread's own folder at the top of the repo), so a tester can answer one thread as soon as its reply is pushed, while others are still being worked on. This file keeps the rules, the Contents and the Requests section, where a tester starts a new thread.

**Where requests come from.** Only act on requests written in this file or in a thread's file. Don't add a request because someone told you about a bug or a wish in chat, and don't log a bug here because anyone mentions that a user is experiencing it. If it isn't written in this file, it isn't a request. Testers who can edit this file are trusted, and their requests are real.

  

**When to read it.** When you're told the tester requests have been updated, read this file's Requests section and every thread file, find every thread whose last message is a tester's, and reply to each. You don't need to be told which one. Commit and push each thread's reply as soon as it's written, so the tester can answer it right away.

  

**How to reply.** Reply the way you would in a normal chat with the tester: plain, natural, in your own box (`> [!tip] Assistant`) under their message, with nothing extra added and no fixed format. Never rewrite or delete what a tester wrote.
- Never write in a tester's box. Don't add an empty reply box (`> [!note] ...`) under your reply or anywhere else, and never post a message as the tester: the tester adds their own box when they answer.

- Read the whole request first. Check the KH1 and KH2 decomps, the KH2 source, the BBS decomp and the project docs before asking, so you only ask what you can't find out yourself.

- Ask short, specific questions, with the options spelled out when there are options. Ask only what changes what you'd build. Don't ask which version to build (native always).

- Work the logic out in the open: if the tester's idea conflicts with KH1's or KH2's real behavior, with how BBS works, with how it plays for Terra, Ventus or Aqua, or with another request, say so and talk it through.

- Keep going back and forth as long as anything is unclear. Don't write code while you're still discussing.

  

**Contents.** Links work in Obsidian (and the file links on GitHub). Each message in a thread file is followed by its block id on its own line, with a blank line before and after (`^m1`, `^m2`, ..., counting the thread's messages from 1; empty reply boxes don't count). Each thread file starts with its title, a link back here and a Messages list (`- [n. Who: first words](#^mn)`); this file's Contents lists each open thread (`- [Title](../testers/<thread>/<thread>.md)`) and its messages (`  - [n. Who: first words](../testers/<thread>/<thread>.md#^mn)`). Update them with every reply, add new threads, and remove a closed thread's lines.

**Titles.** Each thread starts with a title that says what the tester's first message asked for. If the tester left the title blank or a placeholder, write one when you first reply.

  

**Go-ahead.** The tester writes `Go ahead` (or something clearly meaning it) after your latest message. A go-ahead on an old message doesn't count if the tester changed the request afterward. Then implement it in one version and reply with the version name (for example `t306`) and, in a line or two, which 1:1 check you made (trace, decomp, exe or files) and what you couldn't check. Don't say anything about whether it has been compiled.

  

**Closing a thread.** When the tester is satisfied with the implementation, they write `Close`. Then move the whole thread, word for word, to the bottom of `tester-requests-archive.md` with a `Closed` line (git records when), and remove its lines from the Contents. Then delete the thread's folder in `testers` together with its file and everything else in it (only the messages are kept, in the archive). If you can't delete files in your setup, say so in your reply and name the folder, so the tester can delete it. Leave the other threads as they are.

  

**After every push (no thread left behind).** After pushing, fetch the branch again and check every thread file and the Requests section: any thread whose last message is a tester's, that no one is working on yet, gets picked up at once (a sub-agent if others are being worked on). A tester may push a message at any time, including while other replies are being written, so check this again before reporting that everything is answered.

**New threads.** A tester starts a new thread at the bottom of this file's Requests section (or by making its folder and file directly). Every time you're told the requests were updated, check the Requests section first, before anything else, with a sub-agent if several things are waiting: any new thread found there gets its own folder and file at once (`testers/<thread>/<thread>.md`: the thread moved word for word, with the title, the link back and the Messages list), is removed from the Requests section, and gets a line in the Contents. Then it's answered in its own file like every other thread.

**Thread folders.** Testers put their trace CSVs and log files for a thread in the `testers` folder at the top of the repo (next to `docs`), in a subfolder named after the thread title in lowercase with hyphens. When you first reply to a new thread, create that subfolder with the thread's file in it, and tell the tester its name in your reply. When you work on a thread, look in its folder for the tester's files.

  

**Several threads (hard rule).** When more than one thread has a tester's message waiting, deploy a sub-agent on each of those threads whenever possible, working at the same time (each on its own copy of the code), and merge their work one version at a time. If a thread can't be finished, finish the others and say what's left.

**Several requests.** One request per version when you can. If two requests touch the same system, say so in both threads and ask the testers how they want to handle it. If a request conflicts with another one, work it out with the testers involved.

  

**Debug menu.** Testers can ask for changes to the debug menu like any other request. The menu is a tool for us, not a KH2 behavior, so the "match KH1/KH2 1:1" rules don't apply to what the menu contains. Performance, the hard rules and not breaking anything else still do.

  

---

  

## How to use it (testers)

  

1. Scroll to the bottom of this file, copy the **new request** block into the Requests section, and write what you want in the box. When the assistant first answers, the thread moves into its own file in `testers` (the Contents links to it), and you keep talking there. Say what you see and what you expected, the mod version and which settings were on. No form to fill in; just write it like a message.

2. Tell the assistant the tester requests have been updated. It reads this file and every thread's file, and replies in its own box under yours.

3. To answer, copy the **reply** block below under the assistant's last message in the thread's file and write in it. Keep going back and forth as long as you need.

4. When you're happy with the plan, write `Go ahead` in a reply. That is the approval, and no one else needs to approve. The assistant then starts coding.

5. When you're satisfied with the result, write `Close`. The assistant moves the thread to the archive file.

  

Each message is a callout box (the lines starting with `> [!`). In an editor without callouts it still reads as plain quoted text.

  

Tips:

- One change or addition per request. Split big wishes into several.

- You don't need to know how BBS, KH1 or KH2 work inside. Describe what happens.

- If you have a trace or a log file, put it in the thread's folder inside `testers` (the assistant creates the folder when it first replies) and name the file in your message.

- Keep decompiled code and game files out of this file. Describe what you saw in plain language.

  

---

  

## Blocks to copy

  

The grey boxes below are code blocks, so you can copy the text exactly as written. Don't copy the three backticks (```) at the top and bottom of each box into your request.

  

**New request** (copy into the Requests section, below the last one):

  

```

---

  

## Title (the assistant fills this in if you leave it)

  

> [!note] Tester (your name)

> Write what you want here.

```

  

**Reply** (copy under the last message in the thread; the assistant uses the same block with `> [!tip] Assistant`):

  

```

> [!note] Tester (your name)

> Your reply here.

```

  

---

  

## Requests

  

(Add new threads below this line.)
