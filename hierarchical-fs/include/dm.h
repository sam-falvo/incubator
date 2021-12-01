#ifndef DM_DM_H
#define DM_DM_H

#include <stdint.h>


enum {
	E_OK = 0,
	E_NOTIMPL,
	E_NOTEXISTS,
	E_TOOSMALL,
	E_NOMEM,
};

#define S_OK		(E_OK)
#define E_NOTEXIST	(E_NOTEXISTS)
#define E_NOTFOUND	(E_NOTEXISTS)


typedef struct DmAnchorBlock	DmAnchorBlock;
typedef struct DmNode		DmNode;
typedef struct DmQid		DmQid;
typedef struct DmOps		DmOps;


struct DmAnchorBlock {
	int	curdirlen;
	char    *curdir;
	DmNode	*root;
};

struct DmQid {
	uint8_t		type;
	uint32_t	version;
	uint64_t	path;
};


struct DmOps {
};


struct DmNode {
	char		*name;
	DmQid		qid;
	uint32_t	mode;
	uint32_t	atime;
	uint32_t	mtime;
	uint64_t	length;
	char		*owner;
	char		*group;
	char		*muid;
	DmNode		*parent;
	DmNode		*sibling;
	DmNode		*children;
	DmOps		*ops;
	uint32_t	nchildren;
	uint32_t	refcnt;
};


// These definitions are taken from Plan 9's man pages.
// See intro(5) and stat(5) for more details.

#define QTDIR		0x80
#define QTAPPEND	0x40
#define QTEXCL		0x20
#define QTreserved	0x10
#define QTAUTH		0x08
#define QTTMP		0x04

#define DMDIR		(QTDIR << 24)
#define DMAPPEND	(QTAPPEND << 24)
#define DMEXCL		(QTEXCL << 24)
#define DMreserved	(QTreserved << 24)
#define DMAUTH		(QTAUTH << 24)
#define DMTMP		(QTTMP << 24)

#define DMOWNREAD	0x00000100
#define DMOWNWRITE	0x00000080
#define DMOWNEXEC	0x00000040
#define DMGRPREAD	0x00000020
#define DMGRPWRITE	0x00000010
#define DMGRPEXEC	0x00000008
#define DMOTHREAD	0x00000004
#define DMOTHWRITE	0x00000002
#define DMOTHEXEC	0x00000001


int DmChangeDir(DmAnchorBlock *, char *);
int DmCurrentDir(DmAnchorBlock *, char *, int *);
DmAnchorBlock *DmInitialize(void);
void DmExpunge(DmAnchorBlock *);

#endif

