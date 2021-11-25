# Abstract (fill this in last; 4 sentences!)
# Introduction (1pp)
## State my contributions
### List contributions first; SPJ says it drives teh rest of the paper.
### Contributions should be refutable.

# The Problem (1pp)
When writing a new operating system,
it is easy to support a multi-rooted filesystem semantics.
One of the simplest examples is Tripos, which also forms the foundations of AmigaDOS.
The syntax for a complete filename consists of three parts:
the name of a device, assignment, or volume name;
a literal colon character (`:`);
and, everything else that follows the colon.
For example, you might want to save a picture in `Work:Pictures/Schematics/radio.iff`.
The name of the actual file is `radio.iff`,
which is stored in the path `Pictures/Schematics` (with Pictures found in the volume's root directory)
on whatever volume is (physically or logically) named `Work:`.
That's how a human operator would interpret the filename.
Tripos, on the other hand, is *entirely* ignorant of the hierarchical file structures on any given volume.
Instead, all it cares about is a file or directory's *handler*.
Once it knows the handler for a file operation,
it can delegate the task of interpreting the remainder of the path to that handler.
To do this,
Tripos maintains a (volume name, device, handler) mapping dynamically in RAM.
For Tripos, all it cares about is the `Work:` prefix;
after finding a match in the handler table, it now knows what device `Work:` is mounted in, and what its filesystem handler is.
It then delegates all subsequent filesystem operations to this handler,
having passed `Pictures/Schematics/radio.iff` as its parameter to the file-open or file-create operation.

In 1985, Rob Pike and P. J. Weinberger released an essay titled, [*The Hideous Name.*](http://doc.cat-v.org/bell_labs/the_hideous_name/)
In it, Pike, et. al. demonstrated and justified the Unix-style hierarchical name space and its semantics.
Since the release of the essay, system software developers found ways to emulate Unix-style uniform hierarchical naming.
For example, the Atari ST platform saw the release of MiNT,
a Unix-alike operating system that continued to make use of the GEM desktop environment.
In this environment, the `U:` drive would map to the Unix-like file hierarchy.
On the complete opposite end of the spectrum,
Microsoft would update the Windows NT kernel to support arbitrary mount-points with the release of Windows 2000,
allowing Windows users support for more than 26 installed drives.
Particularly cheeky users could, if they were so inclined, mount the floppy drive under `C:\Floppy Disk\A-Drive` if they wanted to.
However, it wasn't until the release of the [Plan 9 from Bell Labs](https://en.wikipedia.org/wiki/Plan_9_from_Bell_Labs) research operating system
that the full potential for a single-rooted namespace would be demonstrated.

It seems that single-rooted filesystem hierarchies are here to stay,
as fewer and fewer new systems use multi-rooted namespaces.
Moreover, per-process name spaces seems highly desirable for any new system.
This motivates any homebrew operating system designer to seriously consider a Unix-style, single-rooted namespace.
The problem is, unlike multi-rooted filesystems, they're surprisingly difficult to implement!

# The Idea (2pp)
To help learn how to implement a single-rooted filesystem hierarchy,
I decided to write my own filesystem library in plain C,
complete with support for mount points.
If time and continued interest permits,
I may attempt to use this library to implement a kind of "structured storage" library.
That application will be documented separately.

I make use of the Carrier-Rider design pattern[1], which is a kind of generalized model-view-controller type pattern.
The file tree structure is the carrier,
a data structure which is shared among a number of different traversal structures
(so-called "cursors", to re-use a term from the field of relational databases),
which take the role of riders.

    .-------.   .-------.
    | Rider |   | Rider |   . . .
    `-------'   `-------'
        |           |
        `----.  .---'
             |  |
             V  V
         .---------.
         | Carrier |
         `---------'

Where possible, my file operations should have the same semantics as Plan 9.
This should ease re-using the software in my own 9P implementations later on.

In the following sub-sections, I use a set of use-cases to evolve the design.
For brevity of documentation, I assume the following type definitions:

    typedef struct fsmounttable fsmounttable;
    typedef struct fsnode       fsnode;
    typedef struct fsops        fsops;
    typedef struct fsqid        fsqid;

Additionally, the objects create a complex graph of references.
To help manage memory, structures often use reference counting.
To keep the examples small and poignant,
**I omit all reference counting logic from the examples.**
The actual implementation, of course, will need proper reference and structure life-cycle management.

## Navigating to the Root Directory

In this case, the programmer would want to refer directly to the root directory (`/`).
There is no tree traversal to perform here;
all we're doing is pointing a cursor at the root.

To describe this, we only need a single `fsnode` structure, representing the root directory itself
and the cursor which points at it.


    struct fsnode {
        char        *name;      // "/" for root directory.
        fsqid       qid;        // 9P Qid fields.
        uint32_t    mode;       // Permissions, file type, etc. flags.
        uint32_t    atime;      // Accessed timestamp.
        uint32_t    mtime;      // Modified timestamp.
        uint64_t    length;     // length of the file, if appropriate.
        char        *owner;     // owning user
        char        *group;     // owning group
        char        *muid;      // user who last modified the file.
    }

    struct fscursor {
        fsnode  *current;
    }

## Listing the Root Directory

We have a reference to the root directory, but what can we do with it?
One obvious application is to list its contents.
To do this, we need to evolve the `fsnode` structure to record other nodes as children.

    struct fsnode {
        char        *name;      // "/" for root directory.
        fsqid       qid;        // 9P Qid fields.
        uint32_t    mode;       // Permissions, file type, etc. flags.
        uint32_t    atime;      // Accessed timestamp.
        uint32_t    mtime;      // Modified timestamp.
        uint64_t    length;     // length of the file, if appropriate.
        char        *owner;     // owning user
        char        *group;     // owning group
        char        *muid;      // user who last modified the file.

        fsnode      *parent;    // Root directory parent points to self.
        fsnode      *sibling;   // Next fsnode in this directory.  NULL for root.
        fsnode      *children;  // NULL for plain files or empty directories.
    }

    struct fscursor {
        fsnode  *current;       // Current fsnode reference

        fsnode  *sibling;       // Directory iterator reference
    }

Now, to list the contents of a directory pointed at by `fscursor.current`,
you need only start at the node pointed at by `children`, and iterate through all of its siblings.

Those familiar with MS-DOS or AmigaDOS will recognize how to start iterating through a directory
via a function named something like "Find First".
The corresponding "Find Next" is used to advance the iterator.
Here's a simple implementation of these two kinds of functions.

    fsnode *
    fs_find_first(fscursor *c, fsnode *n) {
        c->sibling = n->children;
        return c->sibling;
    }

    fsnode *
    fs_find_next(fscursor *c) {
        if(c->sibling) c->sibling = c->sibling->next;
        return c->sibling;
    }

## Walking to the Next Path Element

If we wish to walk to the next path element,
we need to make sure the current `fsnode` is a directory first, and
then iterate through the children to find the correspondingly named child.
This is not hard to do; the use of `fs_find_first` and `fs_find_next` above can be used to make this happen fairly easily.
If we find what we're looking for, just update the cursor's `current` field to point to what you've just found.

Where things get interesting is when you consider the possibility of *mount points*.
We need to check to see if the current directory is a mount point *before* looking through the children.
If it is, then we need to consider the children of the mounted root directory.

The only real change here is in how `fs_find_first` is implemented.
Here, `get_mount_point()` answers with the corresponding root `fsnode` for the mounted file system, given our current directory.

    fsnode *
    fs_find_first(fscursor *c, fsnode *n) {
        fsnode *mountroot = get_mount_point(n);

        if(mountroot != NULL) c->current = mountroot;
        c->sibling = n->children;
        return c->sibling;
    }

To support this, a table is maintained which relates the "old" `fsnode` with a "new" `fsnode`.
The old `fsnode`, of course, refers to a subdirectory in the ancestral file tree,
while the new `fsnode` refers to the root directory of a mounted name space.

    struct fsmounttable {
        int     n;      // How many mount table entries exist.
        fsnode  **old;  // List of old fsnodes.
        fsnode  **new;  // List of new fsnodes, each corresponding to an old fsnode.
    }

    fsnode *
    get_mount_point(fsnode *n) {
        fsmounttable *mt = get_process_mount_table();
        int i;

        for(i = 0; i < mt->n; i++) {
            if(n == old[i]) return new[i];
        }
        return NULL;
    }

## Opening a File, et. al.

Once we have found what we're looking for, and confirmed it has the correct type,
we may "open" it.
The specific behavior taken by opening a file is dependent on the kind of file you're opening.
A disk file, for example, might involve reading in some meta-data from disk storage,
while an RS-232 connection might involve programming a UART with a default set of communications parameters.
We record the specific set of operational methods with the `fsnode` for the file.

    struct fsnode {
        char        *name;      // "/" for root directory.
        fsqid       qid;        // 9P Qid fields.
        uint32_t    mode;       // Permissions, file type, etc. flags.
        uint32_t    atime;      // Accessed timestamp.
        uint32_t    mtime;      // Modified timestamp.
        uint64_t    length;     // length of the file, if appropriate.
        char        *owner;     // owning user
        char        *group;     // owning group
        char        *muid;      // user who last modified the file.
        fsnode      *parent;    // Root directory parent points to self.
        fsnode      *sibling;   // Next fsnode in this directory.  NULL for root.
        fsnode      *children;  // NULL for plain files or empty directories.

        fsops       *ops;       // A set of file-specific methods.
    }

Since each of the standard Plan 9 file operations might need to alter the cursor as well as the file proper,
the operations are defined to work upon a `fscursor`.
Note that the `fsnode` is easily recovered by evaluating `fscursor.current`.

    struct fsops {
        int (*flush)(fscursor *);
        int (*open)(fscursor *, uint8_t mode);
        int (*create)(fscursor *, char *name, uint32_t permissions, uint8_t mode);
        void (*close)(fscursor *);
        int (*read)(fscursor *, uint64_t offset, uint8_t *buf, uint32_t length, uint32_t *actual);
        int (*write)(fscursor *, uint64_t offset, uint8_t *buf, uint32_t length, uint32_t *actual);
        int (*remove)(fscursor *);
        int (*stat)(fscursor *, uint8_t *statbuf, uint32_t maxlen);
        int (*wstat)(fscursor *, uint8_t *statbuf);
    };

To make things a bit more efficient at run-time,
an "open" `fscursor` will cache the `fsops` that is relevant to it.


    struct fscursor {
        fsnode  *current;       // Current fsnode reference
        fsnode  *sibling;       // Directory iterator reference

        fsops   *ops;           // NULL if not yet open.
    }


# The Details (5pp)

Consider the following environment.

        Process mount table:
                |  old  |   new   |
                |-------|---------|
                |  ...  |   ...   |
                | N_DEV | devroot |
                |  ...  |   ...   |


                   .---------------.
        proot ---> | name: ""      |    N_DEV
                   | flags: dir    |   .-----------------.
                   | children: o------>| name: "dev"     |
                   | sibling: NULL |   | flags: dir      |
                   | parent: self  |   | children: NULL  |
                   | ref: 3+       |   | sibling: o---------.
                   `---------------'   | parent: ...     |  |
                                       | ref: 1          |  |
                                       `-----------------'  |
                                   -------------------------'
                                  |    .-----------------.
                                  `--->| name: "bin"     |
                                       | flags: dir      |
                                       | children: o--------- . . .
                                       | sibling: o---------- . . .
                                       | parent: ...     |
                                       | ref: 1          |
                                       `-----------------'
                     .---------------.
        devroot ---> | name: ""      |
                     | flags: dir    |   .-----------------.
                     | children: o------>| name: "cons"    |
                     | sibling: NULL |   | flags: file     |
                     | parent: self  |   | children: NULL  |
                     | ref: 3+       |   | sibling: o---------.
                     `---------------'   | parent: ...     |  |
                                         | ref: 1          |  |
                                         `-----------------'  |
                                     -------------------------'
                                    |    .-----------------.
                                    `--->| name: "kmem"    |
                                         | flags: file     |
                                         | children: NULL  |
                                         | sibling: o---------- . . .
                                         | parent: ...     |
                                         | ref: 1          |
                                         `-----------------'


Let's further suppose we wish to open the file `/dev/cons`.

We start by creating a new `fscursor` which points initially at the root directory `fsnode` (`/`).
The `open` call sees that we want to walk to the `dev` file next,
so it calls `fs_find_first` to start iterating the directory's children.
We see that `/` *is not* a mount point, so we just use `/`'s children in memory.
Eventually `dev` is found, and that now becomes the new current `fsnode` in the cursor.

The `open` call sees that we now want to walk to the `cons` file next.
So, it calls `fs_find_first` again, to start iterating `dev`'s children.
However, `fs_find_first` sees that this directory *is* a mount point,
and takes this opportunity to switch the `fscursor` to `dev`'s root directory (pointed to by `devroot` above).
Now we check its children, and eventually, find that `cons` is listed.
This becomes the current `fsnode` in the cursor.

There's nothing else in the pathname to parse, so `open` now decides to invoke the `open()` method.
If this operation is successful, the kernel initializes the `fscursor` to an "open" state, and returns the referring file descriptor back to the caller.
Otherwise, the `fscursor` remains in the closed state and is dereferenced, potentially causign it to be freed.
An error is returned to the caller.

## Life-Cycle Management

Nodes can be created at any time.
But, at some point, they must also be destroyed,
or else the computer will run out of memory eventually.
Each fsnode will need a reference count to properly keep track of how many items refer to it.

An fsnode's children and sibling pointers are *weak* references; they do not contribute to a fsnode's reference count.
Only the parent pointer, *if* it doesn't refer to itself, contributes towards a reference count.

    struct fsnode {
        char        *name;      // "/" for root directory.
        fsqid       qid;        // 9P Qid fields.
        uint32_t    mode;       // Permissions, file type, etc. flags.
        uint32_t    atime;      // Accessed timestamp.
        uint32_t    mtime;      // Modified timestamp.
        uint64_t    length;     // length of the file, if appropriate.
        char        *owner;     // owning user
        char        *group;     // owning group
        char        *muid;      // user who last modified the file.
        fsnode      *parent;    // Root directory parent points to self.
        fsnode      *sibling;   // Next fsnode in this directory.  NULL for root.
        fsnode      *children;  // NULL for plain files or empty directories.
        fsops       *ops;       // A set of file-specific methods.

	uint32_t    nchildren;	// The number of fsnodes in the children list.
	uint32_t    refcnt;	// Other kinds of reference counts (e.g., handles).
    }

In the structure above,
I split the reference count into two parts:

* nchildren refers to the number of nodes in the children list.  Since each of these nodes must have its parent pointer set to its container, this field effectively counts how many children exist.
* refcnt refers to other kinds of handles, such as file handles opened by an application, mount table entries, etc.

The fsnode is safe for disposal if and only if both nchildren and refcnt are both zero.

Since applications will maintain references to leaves of the fsnode tree, a leaf node's refcnt will be greater than zero for the duration of the file handle's existence.
Transitively, any parent node references it maintains will be accounted for through the nchildren fields of all linked parent nodes.
This stops at the root of the tree.
If the tree is mounted onto another tree, then
the corresponding mount entry in the fsmounttable will have a strong reference to the mount point, itself a leaf of another tree.
And, so, the reference chain can be transitively maintained back to *it's* root, and so on.

A mount record will also maintain a strong reference to the root directory of the mounted tree as well.
Therefore, even if all its children get deallocated, the root will remain as long as the mount point remains.

For this reason, the directory structure maintained by the fsnode hierarchy is best described as a record of all the *live* directory elements;
that is, all the directory nodes which are in active use by someone at the moment.
The structure will be expected to grow and shrink dynamically as program needs dictate.

In order to give the fsnode tree a chance to grow,
we need to provide hooks the `fs_find_first` and `fs_find_next` functions
to allow them the chance to, e.g., read in a directory structure from disk.

    struct fsops {
        int (*flush)(fscursor *);
        int (*open)(fscursor *, uint8_t mode);
        int (*create)(fscursor *, char *name, uint32_t permissions, uint8_t mode);
        void (*close)(fscursor *);
        int (*read)(fscursor *, uint64_t offset, uint8_t *buf, uint32_t length, uint32_t *actual);
        int (*write)(fscursor *, uint64_t offset, uint8_t *buf, uint32_t length, uint32_t *actual);
        int (*remove)(fscursor *);
        int (*stat)(fscursor *, uint8_t *statbuf, uint32_t maxlen);
        int (*wstat)(fscursor *, uint8_t *statbuf);

	int (*find_first)(fscursor *, fsnode *);
	int (*find_next)(fscursor *);
    };

    fsnode *
    fs_find_first(fscursor *c, fsnode *n) {
        fsnode *mountroot = get_mount_point(n);

        if(mountroot != NULL) c->current = mountroot;
        c->sibling = n->children;
	if(!c->sibling) {
		int error = n->ops->find_first(c, n);
		// Somehow handle error here if possible.
	}
        return c->sibling;
    }

    fsnode *
    fs_find_next(fscursor *c) {
        if(c->sibling) c->sibling = c->sibling->next;
	// Just because there's nothing left in memory, it doesn't mean
	// there's nothing left on-disk.  Try to read the rest of the directory
	// from storage.
	if(!c->sibling) {
		int error = n->ops->find_next(c);
		// Somehow handle error here if possible.
	}
        return c->sibling;
    }

Note: As presented here, the `find_first` and `find_next` hooks would be expected to manipulate the fsnode and/or fscursor structures directly.

Observation: You have a choice!  You could use the `fsnode.children` field to maintain a cache of directory entries, or you can just set it to NULL and never think about it again.
In the latter case, `fs_find_first` will immediately defer to `n->ops->find_first()`, which would update the cursor's `c->sibling` field with an fsnode structure
that only has its parent field set appropriately.
When `fs_find_next` is invoked on the cursor, the lack of a sibling will cause `n->ops->find_next()` to be invoked,
in which case you could reuse the fsnode structure (assuming its reference counts indicate it's safe to do!) for the next directory entry.

# Related Work (1-2pp)

Plan 9


# Conclusion/Further Work (0.5-1.0pp)
# References

The references below are listed in the order encountered, not alphabetically.

[1] *BlackBox Tutorial*.  Oberon Microsystems, Inc.  2007.  Accessed 2021 Nov 20.  `https://oberoncore.ru/_media/blackbox/tut-tot.en.pdf`

