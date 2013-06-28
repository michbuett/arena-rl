(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     *
     * @class
     * @name arena.Application
     * @extends alchemy.browser.Application
     * @requires arena.modules.HUD
     * @requires arena.modules.Map
     * @requires arena.modules.Player
     * @requires arena.modules.Renderer
     */
    alchemy.formula.add({
        name: 'arena.Application',
        extend: 'alchemy.browser.Application',

        requires: [
            'arena.modules.HUD',
            'arena.modules.Map',
            'arena.modules.Player',
            'arena.modules.Renderer'
        ],

        overrides: {
            /** @lends arena.Application.prototype */

            config: 'data/app.json',

            modules: [
                'arena.modules.HUD',
                'arena.modules.Map',
                'arena.modules.Player',
                'arena.modules.Renderer'
            ],
        }
    });
}());

