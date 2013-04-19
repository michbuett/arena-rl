
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.view.Factory',
        requires: [
            'arena.view.Prima',
            'arena.view.Map',
            'arena.view.Entity'
        ],

        overrides: {
            viewMap: {
                'arena.Map': 'arena.view.Map',
                'arena.Player': 'arena.view.Entity'
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
                    objName = obj.meta('name');
                }
                return alchemy(this.viewMap[objName] || 'arena.view.Prima');
            }
        }
    });
}());

