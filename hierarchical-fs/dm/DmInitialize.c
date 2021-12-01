#include <stdlib.h>
#include <string.h>

#include <dm.h>


#define DFLT_CURDIR_LEN		32


DmAnchorBlock *
DmInitialize(void) {
	DmAnchorBlock *ab;
	DmNode *n;
	char *curdir;

	curdir = malloc(DFLT_CURDIR_LEN);
	if(!curdir) goto nomem;

	n = malloc(sizeof(DmNode));
	if(!n) goto nomem;

	strcpy(curdir, "/");

	n->name = curdir;
	n->qid.type = QTDIR;
	n->qid.path = 0;
	n->qid.version = 0;
	n->atime = 0;	// Root node is ancient.
	n->mtime = 0;
	n->length = 0;
	n->owner = "root";
	n->group = "root";
	n->muid = "root";
	n->parent = n;
	n->sibling = NULL;
	n->children = NULL;
	n->ops = NULL;	// FIXME: will definitely crash if ever it's used.
	n->nchildren = 0;
	n->refcnt = 1;

	ab = malloc(sizeof(DmAnchorBlock));
	if(ab) {
		memset(ab, 0, sizeof(DmAnchorBlock));

		ab->curdirlen = DFLT_CURDIR_LEN;
		ab->curdir = curdir;
	}
	return ab;

nomem:
	if(n) free(n);
	if(curdir) free(curdir);
	return NULL;
}

