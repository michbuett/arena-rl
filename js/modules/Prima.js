(function () {
    'use strict';

    var alchemy = require('./Alchemy.js');

    /**
     * A generic application module
     *
     * @class
     * @name arena.modules.Prima
     * @extends alchemy.core.Ingredient
     */
    alchemy.formula.add({
        name: 'arena.modules.Prima',
        extend: 'alchemy.core.Ingredient',
        overrides: {
            /** @lends arena.modules.Prima */

            publics: ['prepare', 'update', 'draw'],

            prepare: alchemy.emptyFn,

            update: alchemy.emptyFn,

            draw: alchemy.emptyFn,
        }
    });
}());
