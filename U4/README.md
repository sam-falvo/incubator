# Introduction

Unsuitable V4, also called U4,
is my personal dynamic web presence generator written in Forth.

Unsuitable has gone through several revisions,
but only V1 remains in use today.

* V1 &mdash; currently operating version.
* V2 &mdash; refactored, "productized" revision of V1.  Never released.
* V3 &mdash; experimental features, an attempt at remote authentication.  Never released.
* V4 &mdash; this version.

# Problems with U1 and U2

* **Hard to use.**  Requires intimate knowledge of internal data structures.
* **Web interface is read-only.**  To post new content, you are required to SSH into server and craft customized Forth commands.
* **No preview.**  Creating new content requires backing up and restoring the message base frequently until you're happy with the result.
* **Restricted to blogs.**  U1-U3 were designed when blogs were king.  Today, wikis and micro-blogging sets a new standard for UX.
* **RSS computed from blog content.**  RSS is better exposed as an event log.  Deriving RSS from content works for blogs, but not for wikis.
* **Text optimized.**  The Global Object Store can be used to store binary data; however, there's no way of *referencing* this data.  Thus, useless for storing pictures or other rich content you might want to include in a web page.
* **GOS requires periodic compaction, resizing.**  The Global Object Store is an append-only data structure.

# Problems with U3

* **Incomplete authentication.** Easily subverted.
* **Incomplete implementation.** Never attained feature parity with U1.
* Intended more as a R&D project.

# How U4 Intends to Solve U1-U3 Problems

I might not have all the answers for how to realize these yet,
but these are overriding requirements.

* **Easier to use.**  I aim to reduce the level of internal knowledge needed to administer U4.
* **Read-write web interface.**  I hope to expose some common administration tasks via a web interface, reducing the need to SSH into the web host.
* **Preview support.**  Submit content as drafts, and "publish" them only when you're happy with the output.
* **Blog *and* wiki.**  Blog articles are just wiki articles with greater formality.
* **RSS as event log.**  Articles of all kinds can be uploaded and edited freely without affecting RSS until officially published.
* **Object optimized.**  The new Global Object Store will record additional metadata which allows both text *and* binary data to be retrievable and referenced easily.
* **GOS to be more automated.**  Storage management will be less onerous.
* **Better User Authentication.**  As with U1, read-only operations require no user-auth.  *Each* administrative operation, however, will require a one-time password.
* **Feature parity with U1.**

