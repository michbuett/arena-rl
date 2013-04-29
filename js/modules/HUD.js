/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.modules.HUD',
        extend: 'arena.view.Prima',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.modules.Prima'
        }],
        overrides: {

            prepare: function () {
                var viewId = this.entities.createEntity('hud');

                this.entities.getComponent('view', viewId).on('rendered', function () {
                    this.$fpsEl = $('#fps');
                }, this);
            },

            update: function (params) {
                if (params.frame % 100 === 0) {
                    if (this.$fpsEl) {
                        this.$fpsEl.text('FPS: ' + Math.round(this.app.fps()));
                    }
                }
            }
        }
    });
}());

