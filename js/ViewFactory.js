
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.ViewFactory',
        requires: [
            'arena.View',
            'arena.MapView',
            'arena.EntityView'
        ],

        overrides: {
            viewMap: {
                'arena.Map': 'arena.MapView',
                'arena.Player': 'arena.EntityView'
            },

            init: function () {
            },

            createView: function (obj, cfg) {
                var viewPotion = this.determineViewPotion(obj);
                if (viewPotion) {
                    return viewPotion.brew(cfg);
                } else {
                    throw 'Cannot create view for ' + obj;
                }
            },

            determineViewPotion: function (obj) {
                var objName;
                if (alchemy.isString(obj)) {
                    objName = obj;
                }
                if (alchemy.isObject(obj)) {
                    objName = obj.getMetaAttr('name');
                }
                return alchemy(this.viewMap[objName] || 'arena.View');
            }
        }
    });
}());

