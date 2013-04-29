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

            initEntityTypes: function (types) {
                this.components = {};
                this.entityTypes = types;
            },

            createEntity: function (type, cfg) {
                cfg = cfg || {};
                cfg.id = cfg.id || alchemy.id();

                var components = this.entityTypes[type];
                alchemy.each(components, function (defaults, key) {
                    var collection = this.components[key];
                    if (!collection) {
                        collection = alchemy('Collectum').brew();
                        this.components[key] = collection;
                    }

                    var cmp = alchemy.mix({
                        id: cfg.id,
                        entities: this,
                        messages: this.messages,
                        resources: this.resources
                    }, defaults, cfg[key]);

                    if (cmp.potion) {
                        var potion = alchemy(cmp.potion);
                        delete cmp.potion;
                        cmp = potion.brew(cmp);
                    }
                    collection.add(cmp);
                }, this);

                return cfg.id;
            },

            getComponent: function (entityId, componentKey) {
                var collection = this.components[componentKey];
                return collection && collection.get(entityId);
            }
        }
    });
}());
