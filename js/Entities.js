(function () {
    'use strict';

    var alchemy = require('./Alchemy.js');

    /**
     * The master entity manager
     *
     * @class
     * @name arena.Entities
     * @alias Entities
     * @extends core.alchemy.Oculus
     */
    alchemy.formula.add({
        name: 'arena.Entities',
        alias: 'Entities',
        extend: 'alchemy.core.Oculus',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.modules.Prima'
        }],
        overrides: {
            /** @lends arena.Entities.prototype */

            init: function () {
                this.components = {};
            },

            initEntityTypes: function (types) {
                this.entityTypes = types;
            },

            createEntity: function (type, cfg) {
                cfg = cfg || {};
                var entityId = cfg.id || alchemy.id();
                delete cfg.id;

                var defaults = this.entityTypes[type] || {};
                var componentKeys = alchemy.union(Object.keys(defaults), Object.keys(cfg));

                alchemy.each(componentKeys, function (key) {
                    var collection = this.components[key];
                    if (!collection) {
                        collection = alchemy('Collectum').brew();
                        this.components[key] = collection;
                    }

                    var cmpDefaults = defaults[key];
                    var cmp = alchemy.mix({
                        id: entityId,
                        entities: this,
                        messages: this.messages,
                        resources: this.resources
                    }, cmpDefaults, cfg[key]);

                    if (cmp.potion) {
                        var potion = alchemy(cmp.potion);
                        delete cmp.potion;
                        cmp = potion.brew(cmp);
                    }
                    collection.add(cmp);
                }, this);

                return entityId;
            },

            getComponent: function (componentKey, entityId) {
                var collection;
                collection = this.components[componentKey];
                if (entityId) {
                    return collection && collection.get(entityId);
                } else {
                    return collection;
                }
            }
        }
    });
}());
