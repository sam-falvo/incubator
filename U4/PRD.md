In this document, we use localhost as a hostname for URLs.  However, anything that works for localhost must also work in a hosted environment as well.

PRD.1: Installation and Initial Configuration
tbd.

PRD.2: Operation, Administrator
PRD.2.1: Custom Response Pages
PRD.2.1.1: If a resource cannot be found, a custom 404 error page must be shown instead.
PRD.2.1.2: The default 404 error page must include a link to the home page.

PRD.3: Operation, Author
PRD.3.1: Uploading new articles
PRD.3.1.10: Visiting http://localhost/cgi-bin/blog.fs/edit (with or without the trailing slash) will show an empty edit page.
PRD.3.1.12: Visiting http://localhost/cgi-bin/blog.fs/edit/{n} (where n exists) will show a pre-populated edit page.
PRD.3.1.14: Visiting http://localhost/cgi-bin/blog.fs/edit/{n} (where n doesn't exist) will show a 404 Not Found error.
PRD.3.1.20: The edit page must include a Type field which supports (at a minimum) Blog Article and Wiki Article.
PRD.3.1.22: The edit page must include a Name/Title field.
PRD.3.1.24: The edit page must include an Abstract field.
PRD.3.1.30: The edit page must include a free-form text area labeled Body.
PRD.3.1.40: The edit page must include a body file upload button.
PRD.3.1.50: The edit page must include a submit and cancel button.
PRD.3.1.51: The edit page must have a "published" checkbox.  The checkbox is not checked for new entities.
PRD.3.1.60: U4 must reject edit submissions that don't make sense.  The same edit page is redisplayed with an error.
PRD.3.1.60.1: any submission without a name/title.  This field is used for blog/wiki linking.
PRD.3.1.60.2: any submission missing an abstract, body, and body file.  An empty article is just a waste of resources.
PRD.3.1.60.3: any submission with a body text *and* a body file.
PRD.3.1.60.4: any submission with a name/title that doesn't conform to Wiki syntax guidelines.
PRD.3.1.90: Edits which are accepted results in a preview page, which include the article number for the entity in the URL.
PRD.3.1.91: Preview pages *may* be the same as a pre-filled edit page, as long as a rendered preview appears on the same page.
PRD.3.1.100: While viewing the edit or preview page, if the user presses the cancel button, they are redirected to the home page.

PRD.4: Operation, Reader
PRD.4.1: Reading Index Page
PRD.4.1.1: Visiting http://localhost/cgi-bin/blog.fs (with or without trailing slash) will show the index page.
PRD.4.1.2: The index page can be configured to render a blog index, the wiki's home page, or the blog index embedded within the wiki's home page.
PRD.4.1.3: For a fresh install, links to blog articles are reverse-chronologically sorted (e.g., newer articles are placed on top of older articles).
PRD.4.1.4: For a fresh install, the index page is configured to show a blog index.
PRD.4.1.5: An empty blog index must indicate that no articles exist yet.

PRD.4.2: Reading RSS Page(s)
PRD.4.2.1: Visiting http://localhost/cgi-bin/blog.fs/rss (with or without trailing slash) will show the blog's public RSS feed page.
PRD.4.2.2: Visiting http://localhost/cgi-bin/blog.fs/rss/{anything else not covered in PRD} will result in a 404 Not Found error.
PRD.4.2.3: The RSS feed must never refer or show unpublished or deleted articles.

PRD.4.3: Reading Articles
PRD.4.3.1: Visiting http://localhost/cgi-bin/blog.fs/articles/{n} , where n exists, will show the contents of article {n}.
PRD.4.3.2: Visiting http://localhost/cgi-bin/blog.fs/articles (with or without trailing slash) will result in a 404 Not Found error.
PRD.4.3.3: Visiting http://localhost/cgi-bin/blog.fs/articles/{n}, where n does not exist, will result in a 404 Not Found error.
PRD.4.3.4: Articles will have a link back to the index page, labelled Home.
PRD.4.3.5: Articles which were published *after* another article will have a back-link to the most recently published predecessor article.
PRD.4.3.6: If a previous article has been deleted, an article's back-link to its predecessor must never refer to the deleted article.
PRD.4.3.7: Articles which were published *before* another article will have a forward-link to the earliest published subsequent article.
PRD.4.3.8: If a subsequent article has been deleted, an article's forward-link must never refer to the deleted article.

PRD.4.4: Resolving Unsupported Modules
PRD.4.4.1: Visiting http://localhost/cgi-bin/blog.fs/{anything else not covered in PRD} will result in a 404 Not Found error.


* **Read-write web interface.**  I hope to expose some common administration tasks via a web interface, reducing the need to SSH into the web host.
* **Preview support.**  Submit content as drafts, and "publish" them only when you're happy with the output.
* **RSS as event log.**  Articles of all kinds can be uploaded and edited freely without affecting RSS until officially published.
* **Object optimized.**  The new Global Object Store will record additional metadata which allows both text *and* binary data to be retrievable and referenced easily.
* **Better User Authentication.**  As with U1, read-only operations require no user-auth.  *Each* administrative operation, however, will require a one-time password.

