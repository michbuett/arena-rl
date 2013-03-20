Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.DomContainer
 * @extends Alchemy.v.DomElement
 * @includes Alchemy.v.BasicContainer
 *
 * The type representing an HTML container element
 */
Alchemy.v.DomContainer = {

    cls: 'alc-ct',

    add: function (item) {
        //TODO: move to basic container
        this.items = this.items || [];
        this.items.push(item);

        if (this.dom) {
            item.renderTo(this.dom);
        }
    },

    remove: function (item) {
        //TODO: implement this
        //TODO: move to basic container
    },

    removeAll: function () {
        //TODO: implement this
        //TODO: move to basic container
    },

    update: function () {
        // TODO: move this to BasicContainer
        //console.log('[Alchemy.v.DomContainer] update');
        this.forEachItem(function (item) {
            item.update();
        }, this);
    },

    updateUI: function () {
        this.forEachItem(function (item) {
            item.updateUI();
        }, this);
    },

    /**
     * renders the conatiner's children into the dom element of the container
     */
    renderContent: function (ctxt) {
        ctxt.push('<div class="alc-ct-body">');
        this.forEachItem(function (item) {
            item.render(ctxt);
        }, this);
        ctxt.push('</div>');
        return ctxt;
    },

    linkDom: function () {
        // link dom elements of the container's items
        this.forEachItem(function (item) {
            item.linkDom();
        });
        // link the container's dom
        _super.call(this);
    }
};

Alchemy.v.DomContainer = Alchemy.brew({
    name: 'DomContainer',
    ns: 'Alchemy.v',
    extend: Alchemy.v.DomElement,
    ingredients: [{
        ptype: Alchemy.v.BasicContainer,
        key: 'ct'
    }]
}, Alchemy.v.DomContainer);
