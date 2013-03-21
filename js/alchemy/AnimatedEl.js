(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.alchemy.AnimatedEl',
        extend: 'arena.View',

        overrides: {

            animations: undefined,

            init: function hocuspocus(_super) {
                return function () {
                    var animations = this.animations;
                    this.animations = {};

                    alchemy.each(animations, function (animCfg, animKey) {
                        if (alchemy.isArray(animCfg)) {
                            animCfg = {
                                frames: animCfg
                            };
                        }
                        this.addAnimation(animKey, animCfg);
                    }, this);

                    _super.call(this);
                };
            },

            addAnimation: function (key, cfg) {
                if (this.defaults) {
                    cfg = alchemy.mix(cfg, this.defaults, {
                        override: false
                    });
                }
                cfg.x = cfg.x || 0;
                cfg.y = cfg.y || 0;
                cfg.listeners = {
                    framechanged: {
                        fn: this.clear,
                        scope: this
                    }
                };

                this.animations[key] = alchemy.v.Animation.create(cfg);
            },

            play: function (anim) {
                if (this.animations[anim]) {
                    this.currAnim = anim;
                    this.animations[anim].start();
                }
            },

            getCurrentAnimation: function () {
                return this.animations[this.currAnim];
            },

            clear: function () {
                var renderCtx = this.getContext();
                if (renderCtx) {
                    renderCtx.clearRect(0, 0, this.getWidth(), this.getHeight());
                }
            },

            renderCvsContent: function () {
                var renderCtx = this.getContext(),
                anim = this.getCurrentAnimation();
                if (anim && renderCtx) {
                    anim.render(renderCtx);
                }
            },

            update: function () {
                var anim = this.getCurrentAnimation();
                if (anim) {
                    anim.update();
                }
            }
        }
    });
}());
