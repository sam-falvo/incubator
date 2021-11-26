#ifndef DM_DM_H
#define DM_DM_H


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


typedef struct DmAnchorBlock DmAnchorBlock;

struct DmAnchorBlock {
	int	curdirlen;
	char    *curdir;
};


int DmChangeDir(DmAnchorBlock *, char *);
int DmCurrentDir(DmAnchorBlock *, char *, int *);
DmAnchorBlock *DmInitialize(void);
void DmExpunge(DmAnchorBlock *);

#endif

