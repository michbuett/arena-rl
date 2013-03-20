Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.CvsContainer
 * @extends Alchemy.v.DomElement
 * @includes Alchemy.v.BasicContainer
 *
 * The type representing a canvas container element
 */
Alchemy.v.CvsContainer = {
    constructor: function (cfg) {
        cfg = Alchemy.mix(cfg || {}, {
            tag: 'canvas'
        });

        _super.call(this, cfg);
    },

    getContext: function () {
        if (!this.renderCtx && this.dom) {
            this.renderCtx = this.dom.getContext('2d');
            //this.renderCtx.scale(2, 2);
        }
        return this.renderCtx;
    },

    linkDom: function (el) {
        _super.call(this, el);
        this.renderCvsContent();
    },

    renderAttributes: function (ctxt) {
        ctxt.push(' width="', this.getWidth(), 'px" height="', this.getHeight(), 'px"');
        return ctxt;
    },

    renderCvsContent: function () {
        var renderCtx = this.getContext();
        if (renderCtx) {
            //renderCtx.clearRect(0, 0, this.getWidth(), this.getHeight());
            this.forEachItem(function (item) {
                item.render(renderCtx);
            });
        }
    },

    add: function (item) {
        this.items = this.items || [];
        this.items.push(item);

        var renderCtx = this.getContext();
        if (renderCtx) {
            item.render(renderCtx);
        }
    },

    update: function () {
        // TODO: move this to BasicContainer
        //console.log('[Alchemy.v.CvsContainer] update');
        var items = this.items,
            i;
        if (items && items.length > 0) {
            for (i = 0; i < items.length; i++) {
                items[i].update();
            }
        }
    },

    updateUI: function () {
        this.renderCvsContent();
    }
};

Alchemy.v.CvsContainer = Alchemy.brew({
    extend: Alchemy.v.DomElement,
    ingredients: [{
        ptype: Alchemy.v.BasicContainer,
        key: 'ct'
    }]
}, Alchemy.v.CvsContainer);
