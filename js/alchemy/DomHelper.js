Alchemy.ns('Alchemy.v');

Alchemy.v.DomHelper = {
    getBody: function () {
        var doc = document;
        return doc.body || doc.documentElement;
    },

    get: function (selector, root) {
        root = root || document;
        return root.querySelector(selector);
    },

    getAll: function (selector, root) {
        root = root || document;
        return root.querySelectorAll(selector);
    },

    /**
     * @return {Number}
     *      the current width of the browser window
     */
    getWindowWidth: function () {
        return window.innerWidth || document.documentElement.clientHeight || document.body.clientHeight;
    },

    /**
     * @return {Number}
     *      the current height of the browser window
     */
    getWindowHeight: function () {
        return window.innerHeight || document.documentElement.clientHeight || document.body.clientHeight;
    },

    /**
     * removes a HTMLElement from the DOM tree
     *
     * @param {HTMLElement} el
     *      the dom node to be removed
     */
    removeNode: function (el) {
        var parentEl = el.parentElement
        if (parentEl) {
            parentEl.removeChild(el);
        }
    }
};

Alchemy.v.DomHelper = Alchemy.brew({}, Alchemy.v.DomHelper);
