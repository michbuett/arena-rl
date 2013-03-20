Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.BasicContainer
 * @extends Alchemy.MateriaPrima
 *
 * A simple mixin providing container functionality
 */
Alchemy.v.BasicContainer = {

    publics: [
        'items',
        'addItem',
        'removeItem',
        'getById',
        'forEachItem'
    ],

    /**
     * @cfg {Array} items
     * the set of child elements
     */
    items: undefined,

    addItem: function () {
        throw 'Method not implemented';
    },

    removeItem: function () {
        throw 'Method not implemented';
    },

    getById: function (id) {
        var items = this.items,
            item,
            i, l;
        if (items && items.length > 0) {
            for (i = 0, l = items.length; i < l; i++) {
                item = items[i];
                if (item.id == id) {
                    return item;
                }
            }
        }
        return null;
    },

    /**
     * @param {Function} callback
     *      the method which should be called for each item; callback is invoked
     *      with three arguments: the value of the element, the index of the
     *      element, and the Array object being traversed.
     *
     * @param {Object} scope
     *      the executeion scope for the callback
     */
    forEachItem: function (callback, scope) {
        var items = this.items;
        if (items && items.length > 0) {
            items.forEach(callback, scope);
        }
    }
};

Alchemy.v.BasicContainer = Alchemy.brew({
    name: 'BasicContainer',
    ns: 'Alchemy.v',
    extend: Alchemy.Ingredient
}, Alchemy.v.BasicContainer);
