(function () {
    'use strict';

    var alchemy = require('./Alchemy.js');

    /**
     * A generic application module
     *
     * @class
     * @name arena.ApplicationModule
     * @extends alchemy.core.Ingredient
     */
    alchemy.formula.add({
        name: 'arena.ApplicationModule',
        extend: 'alchemy.core.Ingredient',
        overrides: {
            /** @lends arena.ApplicationModule */

            publics: ['prepare', 'update', 'draw'],

            prepare: alchemy.emptyFn,

            update: alchemy.emptyFn,

            draw: alchemy.emptyFn,
        }
    });
}());
